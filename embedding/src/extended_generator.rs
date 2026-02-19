use anyhow::{Context, Result, anyhow};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{info, warn};
use crate::generator::Generator;

/// Конфигурация для подключения к внешнему inference-серверу (llama.cpp, Ollama, vLLM).
/// Сервер должен предоставлять OpenAI-совместимый API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedGeneratorConfig
{
    /// Базовый URL сервера (например "http://localhost:8080")
    #[serde(default = "default_base_url")]
    pub base_url: String,
    /// Имя модели (для OpenAI API, llama.cpp игнорирует это поле)
    #[serde(default = "default_model")]
    pub model: String,
    /// Температура генерации (0.0 = greedy, 1.0 = creative)
    #[serde(default = "default_temperature")]
    pub temperature: f64,
    /// Nucleus sampling (top-p)
    #[serde(default = "default_top_p")]
    pub top_p: f64,
    /// Top-K sampling
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    /// Максимальное количество токенов в ответе
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
    /// Штраф за повторения
    #[serde(default = "default_repeat_penalty")]
    pub repeat_penalty: f32,
}

fn default_base_url() -> String { "http://localhost:8080".to_owned() }
fn default_model() -> String { "local-model".to_owned() }
fn default_temperature() -> f64 { 0.6 }
fn default_top_p() -> f64 { 0.9 }
fn default_top_k() -> usize { 30 }
fn default_max_tokens() -> usize { 1000 }
fn default_repeat_penalty() -> f32 { 1.1 }

impl Default for ExtendedGeneratorConfig
{
    fn default() -> Self
    {
        Self
        {
            base_url: default_base_url(),
            model: default_model(),
            temperature: default_temperature(),
            top_p: default_top_p(),
            top_k: default_top_k(),
            max_tokens: default_max_tokens(),
            repeat_penalty: default_repeat_penalty(),
        }
    }
}

// --- OpenAI-compatible API request/response types ---

#[derive(Serialize)]
struct ChatCompletionRequest
{
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f64,
    top_p: f64,
    top_k: usize,
    max_tokens: usize,
    repeat_penalty: f32,
    stream: bool,
}

#[derive(Serialize, Clone)]
struct ChatMessage
{
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionChunk
{
    choices: Vec<ChunkChoice>,
}

#[derive(Deserialize)]
struct ChunkChoice
{
    delta: Option<ChunkDelta>,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct ChunkDelta
{
    content: Option<String>,
}

/// Клиент для генерации текста через внешний OpenAI-совместимый сервер.
///
/// Поддерживает llama.cpp server, Ollama, vLLM и любой другой сервер
/// с OpenAI-совместимым API (`/v1/chat/completions`).
///
/// Преимущества перед локальным Generator:
/// - Сервер обрабатывает множество запросов одновременно (continuous batching)
/// - Не блокирует RwLock в вашем приложении
/// - Стриминг токенов через SSE
#[derive(Clone)]
pub struct ExtendedGenerator
{
    client: reqwest::Client,
    config: ExtendedGeneratorConfig,
    system_prompt: String,
    is_connected: bool,
}

impl ExtendedGenerator
{
    pub fn new(config: ExtendedGeneratorConfig) -> Self
    {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .unwrap_or_default();

        Self
        {
            client,
            config,
            system_prompt: "Ты - ассистент RAG системы, ты должен отвечать на вопросы пользователей \
                используя ТОЛЬКО предоставленный контекст для формирования ответа. Начало ответа \
                должно звучать так: `На основе имеющейся у меня информации: {далее идет твой ответ}`. \
                Если в контексте нет информации, скажи: \"Не могу ответить на основе имеющейся информации\""
                .to_owned(),
            is_connected: false,
        }
    }

    pub fn with_system_prompt(mut self, prompt: String) -> Self
    {
        self.system_prompt = prompt;
        self
    }

    pub fn get_system_prompt(&self) -> &str
    {
        &self.system_prompt
    }

    /// Проверяет доступность сервера
    pub async fn health_check(&self) -> bool
    {
        let url = format!("{}/health", self.config.base_url);
        match self.client.get(&url).send().await
        {
            Ok(resp) => resp.status().is_success(),
            Err(_) =>
            {
                // Некоторые серверы не имеют /health, пробуем /v1/models
                let url = format!("{}/v1/models", self.config.base_url);
                self.client.get(&url).send().await
                    .map(|r| r.status().is_success())
                    .unwrap_or(false)
            }
        }
    }

    /// Генерирует ответ на основе запроса и контекста, стримя токены через sender.
    ///
    /// # Аргументы
    /// * `query` - Вопрос пользователя
    /// * `context` - Контекст из RAG (результаты поиска)
    /// * `sender` - Канал для стриминга токенов
    pub async fn prompt<C: ToString>(
        &self,
        query: &str,
        context: &[C],
        sender: UnboundedSender<String>,
    ) -> Result<()>
    {
        let context_text = context.iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        let user_message = format!("Контекст:\n{}\n\nВопрос: {}", context_text, query);

        let messages = vec![
            ChatMessage { role: "system".to_owned(), content: self.system_prompt.clone() },
            ChatMessage { role: "user".to_owned(), content: user_message },
        ];

        let request_body = ChatCompletionRequest
        {
            model: self.config.model.clone(),
            messages,
            temperature: self.config.temperature,
            top_p: self.config.top_p,
            top_k: self.config.top_k,
            max_tokens: self.config.max_tokens,
            repeat_penalty: self.config.repeat_penalty,
            stream: true,
        };

        let url = format!("{}/v1/chat/completions", self.config.base_url);

        info!("Sending streaming request to {}", url);
        let start = std::time::Instant::now();

        let response = self.client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .context("Ошибка подключения к inference-серверу")?;

        if !response.status().is_success()
        {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Inference-сервер вернул ошибку {}: {}", status, body));
        }

        // Парсим SSE поток
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut token_count = 0usize;

        while let Some(chunk) = stream.next().await
        {
            let chunk = chunk.context("Ошибка чтения потока от inference-сервера")?;
            let text = String::from_utf8_lossy(&chunk);
            buffer.push_str(&text);

            // Обрабатываем все полные SSE-строки в буфере
            while let Some(line_end) = buffer.find('\n')
            {
                let line = buffer[..line_end].trim().to_owned();
                buffer = buffer[line_end + 1..].to_owned();

                if line.is_empty() || line.starts_with(':')
                {
                    continue;
                }

                if let Some(data) = line.strip_prefix("data: ")
                {
                    let data = data.trim();

                    if data == "[DONE]"
                    {
                        info!("Генерация завершена: {} токенов за {:?}", token_count, start.elapsed());
                        return Ok(());
                    }

                    match serde_json::from_str::<ChatCompletionChunk>(data)
                    {
                        Ok(chunk) =>
                        {
                            for choice in &chunk.choices
                            {
                                if let Some(delta) = &choice.delta
                                {
                                    if let Some(content) = &delta.content
                                    {
                                        if !content.is_empty()
                                        {
                                            token_count += 1;
                                            if sender.is_closed()
                                            {
                                                warn!("Операция отменена пользователем");
                                                return Ok(());
                                            }
                                            sender.send(content.clone())
                                                .map_err(|e| anyhow!("Ошибка отправки токена: {}", e))?;
                                        }
                                    }
                                }

                                if choice.finish_reason.is_some()
                                {
                                    info!("Генерация завершена (finish_reason): {} токенов за {:?}",
                                        token_count, start.elapsed());
                                    return Ok(());
                                }
                            }
                        }
                        Err(e) =>
                        {
                            warn!("Не удалось распарсить SSE chunk: {} (data: {})", e, data);
                        }
                    }
                }
            }
        }

        info!("Поток завершён: {} токенов за {:?}", token_count, start.elapsed());
        Ok(())
    }

    /// Генерирует ответ без стриминга (весь ответ целиком).
    pub async fn prompt_sync<C: ToString>(
        &self,
        query: &str,
        context: &[C],
    ) -> Result<String>
    {
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();

        self.prompt(query, context, sender).await?;

        let mut result = String::new();
        while let Some(token) = receiver.recv().await
        {
            result.push_str(&token);
        }

        Ok(result)
    }
}

impl Generator for ExtendedGenerator
{
    /// Проверяет доступность сервера и кэширует результат в is_connected.
    fn load_model(&mut self) -> impl std::future::Future<Output = Result<()>> + Send
    {
        async move
        {
            self.is_connected = self.health_check().await;
            if self.is_connected
            {
                Ok(())
            }
            else
            {
                Err(anyhow!("External inference server is not reachable at {}", self.config.base_url))
            }
        }
    }

    fn unload_model(&mut self)
    {
        self.is_connected = false;
    }

    /// Возвращает кэшированное состояние — актуально после load_model().
    fn model_is_loaded(&self) -> bool
    {
        self.is_connected
    }

    /// Модель находится на удалённом сервере, поэтому размер неизвестен.
    fn get_size_in_bytes(&self) -> usize
    {
        0
    }

    fn get_system_prompt(&self) -> &str
    {
        &self.system_prompt
    }

    /// Блокирующий мост к async prompt.
    ///
    /// Трейт требует синхронного `fn prompt`, но логика — HTTP-запрос.
    /// `block_in_place` паркует текущий поток tokio и позволяет безопасно
    /// вызвать `block_on` внутри уже выполняющегося рантайма.
    /// Вызов `self.prompt(...)` разрешается в inherent async-метод
    /// (приоритет inherent > trait при разрешении имён).
    fn prompt<'a, C: ToString + AsRef<str>>(
        &'a mut self,
        query: &'a str,
        context: &'a [C],
        sender: tokio::sync::mpsc::UnboundedSender<String>,
    ) -> Result<()>
    {
        tokio::task::block_in_place(||
        {
            tokio::runtime::Handle::current().block_on(
                ExtendedGenerator::prompt(self, query, context, sender)
            )
        })
    }
}


// prompt() — async, не нужен &mut self, не блокирует ничего
// 100 concurrent вызовов prompt() спокойно уходят на сервер параллельно
// Не нужен RwLock, не нужен std::thread::spawn
// Инструкция по развёртыванию llama.cpp server
// 1. Сборка llama.cpp

// # CPU
// git clone https://github.com/ggml-org/llama.cpp
// cd llama.cpp
// cmake -B build
// cmake --build build --config Release -j$(nproc)

// # С CUDA (если есть GPU)
// cmake -B build -DGGML_CUDA=ON
// cmake --build build --config Release -j$(nproc)
// 2. Запуск сервера

// ./build/bin/llama-server \
//     -m /home/phobos/projects/rust/law-rag/model/generator/llama3/model-q4_K.gguf \
//     --host 0.0.0.0 \
//     --port 8080 \
//     -ngl 99 \
//     -c 8192 \
//     --parallel 8 \
//     -t 8
// Параметры:

// Флаг	Значение
// -m	Путь к твоей GGUF модели
// -ngl 99	Выгрузить все слои на GPU (если есть). Для CPU — убрать
// -c 8192	Размер контекста
// --parallel 8	Количество слотов для concurrent запросов
// -t 8	Количество CPU потоков
// 3. Проверка

// curl http://localhost:8080/health
// # {"status":"ok"}

// curl http://localhost:8080/v1/chat/completions \
//   -H "Content-Type: application/json" \
//   -d '{"messages":[{"role":"user","content":"Привет"}],"stream":false,"max_tokens":50}'
// 4. Использование в коде

// use embedding::{ExtendedGenerator, ExtendedGeneratorConfig};

// let config = ExtendedGeneratorConfig {
//     base_url: "http://localhost:8080".to_owned(),
//     ..Default::default()
// };
// let generator = ExtendedGenerator::new(config);

// // Проверка доступности
// assert!(generator.health_check().await);

// // Стриминг
// let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
// tokio::spawn(async move {
//     generator.prompt("Вопрос", &context, sender).await.unwrap();
// });
// while let Some(token) = receiver.recv().await {
//     print!("{}", token);
// }
// 5. Настройка --parallel
// --parallel — количество одновременных генераций. Для 100 пользователей:

// GPU: --parallel 8-16 (ограничено VRAM)
// CPU: --parallel 4-8 (ограничено ядрами)
// Запросы сверх лимита встают в очередь внутри llama.cpp сервера автоматически. Это гораздо эффективнее чем блокировка через RwLock.