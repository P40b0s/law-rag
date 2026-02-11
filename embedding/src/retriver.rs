use std::{ops::Deref, path::{Path, PathBuf}, sync::{Arc, LazyLock, OnceLock}};
use crate::{EmbeddingConfiguration, model::{Model, ModelName}};
use anyhow::{Context, Result, anyhow};
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::{bert::{BertModel, Config, DTYPE}};
use rag_core::Encoder;
use serde::{Deserialize, Serialize};
use tokenizers::{PaddingParams, Tokenizer};
use tracing::{debug, error, info, warn};
//static MODEL: OnceLock<BertModel> = OnceLock::<BertModel>::new();



/// Модель для получения векторных представлений (embeddings) текста.
///
/// Использует BERT-подобную архитектуру для генерации плотных векторных представлений текстов,
/// которые затем используются для семантического поиска и ранжирования документов.
///
/// # Архитектура
/// - Токенизация текста с padding до BatchLongest
/// - Прямой проход через BERT модель
/// - Mean pooling с учетом attention mask для получения одного вектора на текст
/// - L2 нормализация для косинусного сходства
pub struct RetriverModel
{
    /// Размерность выходного вектора эмбеддинга
    pub dimension: usize,
    /// Максимальное количество токенов в одном чанке (обычно 8192)
    pub max_tokens: usize,
    /// Количество токенов перекрытия между чанками для сохранения контекста
    pub overlap_tokens: usize,
    /// Имя используемой модели
    pub model_name: ModelName,
    /// Устройство для вычислений (CPU или CUDA)
    device: Device,
    /// Конфигурация BERT модели
    config: Config,
    /// Настройки путей к файлам модели
    embedding_config: Arc<EmbeddingConfiguration>,
    /// Токенизатор для преобразования текста в токены
    tokenizer: Tokenizer,
    /// Загруженная BERT модель (None если не загружена)
    model: Option<BertModel>
}

impl Encoder for RetriverModel
{
    fn decode(&self, ids: &[u32], skip_special_tokens: bool) -> anyhow::Result<String> 
    {
        self.tokenizer.decode(ids, skip_special_tokens)
            .map_err(|e| anyhow!("Decode error {}", e))
    }

    fn encode(&self, text: &str, add_special_tokens: bool) -> anyhow::Result<tokenizers::Encoding> 
    {
        self.tokenizer.encode(text, add_special_tokens)
            .map_err(|e| anyhow!("Encode error {}", e))
    }

    fn max_tokens(&self) -> usize 
    {
        self.max_tokens
    }

    fn overlap_tokens(&self) -> usize 
    {
        self.overlap_tokens
    }

    fn tokenizer(&self) -> &tokenizers::Tokenizer 
    {
        &self.tokenizer
    }
}


impl RetriverModel
{
    /// Создает новый экземпляр RetriverModel.
    ///
    /// # Процесс инициализации:
    ///
    /// 1. **Выбор устройства**: Автоматически использует CUDA (GPU) если доступна, иначе CPU
    /// 2. **Загрузка токенизатора**: Читает tokenizer.json из конфигурации
    /// 3. **Настройка padding**: BatchLongest - выравнивание до самой длинной последовательности в батче
    /// 4. **Загрузка конфигурации модели**: config.json с параметрами архитектуры BERT
    ///
    /// # Аргументы
    /// * `emb_config` - Конфигурация с путями к файлам модели и токенизатора
    ///
    /// # Возвращает
    /// * `Ok(RetriverModel)` - Инициализированная модель (сама BERT модель еще не загружена)
    /// * `Err` - Если не удалось загрузить токенизатор или конфигурацию
    ///
    /// # Примечание
    /// После создания нужно вызвать `load_model()` для загрузки весов BERT модели в память
    pub async fn new(emb_config: Arc<EmbeddingConfiguration>) -> Result<Self>
    {
        info!("Try load {}", emb_config.retriver_tokenizer_path.display());
        let device = Device::cuda_if_available(0).unwrap_or(Device::Cpu);
        info!("Running on device: {:?}", &device);
        let max_tokens = 8192;
        

        // Загрузка и десериализация токенизатора
        let tokenizer_file = &emb_config.retriver_tokenizer_path;
        let tokenizer = tokio::fs::read(&tokenizer_file).await
            .context(format!("Error load tokenizer file from {}", tokenizer_file.display()))?;
        let mut tokenizer = Tokenizer::from_bytes(tokenizer)
            .map_err(|e| anyhow::anyhow!("Error deserialize tokenizer file {}", e))?;

        // Настройка padding: BatchLongest для эффективной обработки батчей разной длины
        let pp = PaddingParams
        {
            strategy: tokenizers::PaddingStrategy::BatchLongest,
            ..Default::default()
        };
        tokenizer.with_padding(Some(pp));

        // Загрузка конфигурации BERT модели
        let model_config_file = &emb_config.retriver_config_path;
        let model_config = tokio::fs::read(&model_config_file).await
            .context(format!("Error load config file from {}", model_config_file.display()))?;
        let config = serde_json::from_slice(&model_config).context("Error deserialize config file")?;

        Ok(Self
        {
            dimension: 1024, // Обычно для BERT hidden_size = 1024, но может отличаться в зависимости от модели
            max_tokens,
            overlap_tokens: (max_tokens / 10).min(256), // 10% от max_tokens, но не более 256
            model_name: ModelName::M3,
            config,
            embedding_config: emb_config,
            device,
            tokenizer,
            model: None // Модель будет загружена при вызове load_model()
        })
    }

    /// Генерирует вектор эмбеддингов для массива текстов.
    ///
    /// # Аргументы
    /// * `texts` - Слайс строк для генерации эмбеддингов
    ///
    /// # Возвращает
    /// * `Ok(Vec<f32>)` - Вектор эмбеддингов (размерность = self.dimension)
    /// * `Err` - Если произошла ошибка или количество текстов != 1
    ///
    /// # Примечание
    /// Функция ожидает ровно один текст во входном массиве.
    /// Для батчевой обработки используйте `embed_tensor_batch` напрямую.
    pub async fn generate_embeddings(&self, texts: &[&str]) -> Result<Vec<f32>>
    {
        // Валидация входных данных
        if texts.is_empty() {
            return Err(anyhow!("Cannot generate embeddings for empty text array"));
        }

        let text_len = texts[0].len();
        info!("Generating embeddings for text of {} bytes ({} texts in batch)", text_len, texts.len());

        // Предупреждение о слишком длинных текстах
        if text_len > 1_000_000 {  // 1MB
            warn!("Very large text ({} bytes), this may take a long time", text_len);
        }

        let start = std::time::Instant::now();
        let tensor = self.embed_tensor_batch(texts).await?;
        info!("Tensor generation took {:?}", start.elapsed());

        let mut vector = tensor.to_vec2()?;
        debug!("emb vector len: {}", vector.len());

        if vector.len() > 1
        {
            return Err(anyhow!("Vector Vec<Vec<f32>> is empty or bigger than 1"));
        }
        let first = vector.pop();
        if let Some(first) = first
        {
            info!("Successfully generated embedding vector of dimension {}", first.len());
            Ok(first)
        }
        else
        {
            return Err(anyhow!("Vector Vec<Vec<f32>> is empty or bigger than 1"));
        }
    }
    /// Генерирует тензор эмбеддингов для батча текстов.
    ///
    /// # Процесс генерации:
    ///
    /// 1. **Токенизация**: Преобразование текстов в последовательности ID токенов
    ///    - Используется padding до BatchLongest (выравнивание по самой длинной последовательности)
    ///    - Генерируется attention_mask: 1 для реальных токенов, 0 для padding
    ///
    /// 2. **Создание тензоров**:
    ///    - token_ids: ID токенов для каждого текста
    ///    - attention_mask: маска для игнорирования padding токенов
    ///    - token_type_ids: тип токена (для BERT, обычно все нули для одного сегмента)
    ///
    /// 3. **Прямой проход через BERT**: Получение контекстуализированных эмбеддингов
    ///    - Вход: [batch_size, seq_len]
    ///    - Выход: [batch_size, seq_len, hidden_size] (обычно hidden_size = 1024)
    ///
    /// 4. **Mean pooling**: Агрегация в один вектор на текст (см. `mean_pooling`)
    ///    - Выход: [batch_size, hidden_size]
    ///
    /// # Аргументы
    /// * `texts` - Слайс текстов для обработки
    ///
    /// # Возвращает
    /// Тензор формы [batch_size, hidden_size] с нормализованными эмбеддингами
    async fn embed_tensor_batch(&self, texts: &[&str]) -> Result<Tensor>
    {
        let start = std::time::Instant::now();
        let device = self.device();
        let model = self.model()?;

        info!("Starting tokenization for {} texts", texts.len());
        let tokenize_start = std::time::Instant::now();

        // Шаг 1: Токенизация всех текстов в батче
        let tokens = Model::tokenizer(self)
            .encode_batch(texts.to_vec(), true)
            .map_err(|e| anyhow!("Encode error {}", e))?;

        info!("Tokenization took {:?}, {} tokens in first sequence",
            tokenize_start.elapsed(),
            tokens.first().map(|t| t.get_ids().len()).unwrap_or(0));

        let tensor_prep_start = std::time::Instant::now();

        // Шаг 2: Создание тензоров token_ids для каждого текста
        // Преобразуем Vec<u32> (ID токенов) в Tensor и собираем в Vec<Tensor>
        let token_ids = tokens
            .iter()
            .map(|tokens|
            {
                let tokens = tokens.get_ids().to_vec();
                Ok(Tensor::new(tokens.as_slice(), device)?)
            })
            .collect::<Result<Vec<_>>>()?;

        // Шаг 3: Создание тензоров attention_mask
        // 1 = реальный токен, 0 = padding токен
        let attention_mask = tokens
            .iter()
            .map(|tokens|
            {
                let tokens = tokens.get_attention_mask().to_vec();
                Ok(Tensor::new(tokens.as_slice(), device)?)
            })
            .collect::<Result<Vec<_>>>()?;

        // Шаг 4: Объединение отдельных тензоров в батч
        // Vec<Tensor[seq_len]> -> Tensor[batch_size, seq_len]
        let token_ids = Tensor::stack(&token_ids, 0)?;
        let attention_mask = Tensor::stack(&attention_mask, 0)?;

        // Создаем token_type_ids (все нули для single-segment задач)
        let token_type_ids = token_ids.zeros_like()?;

        info!("Tensor preparation took {:?}, shape: {:?}", tensor_prep_start.elapsed(), token_ids.shape());

        // Проверка размера входных данных
        let shape = token_ids.shape();
        if shape.dims().len() >= 2 {
            let seq_len = shape.dims()[1];
            if seq_len > self.max_tokens {
                warn!("Sequence length {} exceeds max_tokens {}, this may cause issues",
                    seq_len, self.max_tokens);
            }
        }

        info!("Starting model forward pass with shape {:?}", token_ids.shape());
        let forward_start = std::time::Instant::now();

        // Шаг 5: Прямой проход через BERT модель
        // [batch, seq_len] -> [batch, seq_len, hidden_size]
        let embeddings = model.forward(&token_ids, &token_type_ids, Some(&attention_mask))?;
        info!("Model forward pass took {:?}, generated embeddings {:?}",
            forward_start.elapsed(), embeddings.shape());

        let pooling_start = std::time::Instant::now();

        // Шаг 6: Mean pooling для получения одного вектора на текст
        // [batch, seq_len, hidden] -> [batch, hidden]
        let embeddings = Self::mean_pooling(&embeddings, &attention_mask)?;
        info!("Mean pooling took {:?}, pooled embeddings {:?}",
            pooling_start.elapsed(), embeddings.shape());

        info!("Total embedding generation took {:?}", start.elapsed());

        Ok(embeddings)
    }

    /// Выполняет mean pooling эмбеддингов с учетом attention mask и L2 нормализацию.
    ///
    /// # Что происходит внутри:
    ///
    /// 1. **Подготовка маски** (`unsqueeze(2)`):
    ///    - Исходная форма attention_mask: [batch_size, seq_len]
    ///    - После unsqueeze: [batch_size, seq_len, 1]
    ///    - Это позволяет применить маску к каждой размерности эмбеддинга
    ///
    /// 2. **Вычисление количества реальных токенов** (`sum(1)`):
    ///    - sum_mask: сумма по измерению 1 (seq_len)
    ///    - Результат: [batch_size, 1] - сколько не-padding токенов в каждом примере
    ///
    /// 3. **Усреднение эмбеддингов** (mean pooling):
    ///    - Умножаем embeddings на маску: зануляем padding токены
    ///    - Суммируем по seq_len: складываем все токены в один вектор
    ///    - Делим на sum_mask: получаем среднее только по реальным токенам
    ///
    /// 4. **L2 нормализация**:
    ///    - Вычисляем L2 норму: sqrt(sum(x²))
    ///    - Делим каждый элемент на норму
    ///    - Результат: вектор единичной длины (для косинусного сходства)
    ///
    /// # Аргументы
    /// * `embeddings` - Тензор эмбеддингов формы [batch_size, seq_len, hidden_size]
    /// * `attention_mask` - Маска внимания формы [batch_size, seq_len], где 1 = реальный токен, 0 = padding
    ///
    /// # Возвращает
    /// Нормализованный тензор формы [batch_size, hidden_size]
    fn mean_pooling(embeddings: &Tensor, attention_mask: &Tensor) -> Result<Tensor>
    {
        // Шаг 1: Преобразуем маску к типу float и добавляем размерность для broadcasting
        // [batch, seq_len] -> [batch, seq_len, 1]
        let attention_mask_for_pooling = attention_mask.to_dtype(DTYPE)?.unsqueeze(2)?;

        // Шаг 2: Вычисляем количество реальных (не padding) токенов в каждом примере
        // [batch, seq_len, 1] -> [batch, 1] (сумма по измерению 1)
        let sum_mask = attention_mask_for_pooling.sum(1)?;

        // Шаг 3: Mean pooling
        // Умножаем эмбеддинги на маску (зануляем padding) и суммируем по seq_len
        // [batch, seq_len, hidden] * [batch, seq_len, 1] -> [batch, seq_len, hidden]
        // sum(1) -> [batch, hidden]
        let embeddings = (embeddings.broadcast_mul(&attention_mask_for_pooling)?).sum(1)?;

        // Делим на количество реальных токенов (получаем среднее)
        // [batch, hidden] / [batch, 1] -> [batch, hidden]
        let embeddings = embeddings.broadcast_div(&sum_mask)?;

        // Шаг 4: L2 нормализация
        // Вычисляем норму: sqrt(sum(x²)) и делим каждый элемент на неё
        // Результат: вектор единичной длины (||v|| = 1)
        let embeddings = embeddings.broadcast_div(&embeddings.sqr()?.sum_keepdim(1)?.sqrt()?)?;

        Ok(embeddings)
    }

    /// Генерирует тензор эмбеддингов для одного текста.
    ///
    /// Упрощенная версия `embed_tensor_batch` для обработки одиночного текста.
    /// Для батчевой обработки используйте `embed_tensor_batch`.
    #[allow(dead_code)]
    async fn embed_tensor(&self, text: &str) -> Result<Tensor>
    {
        let start = std::time::Instant::now();
        let model = self.model()?;
        let tokens = Model::tokenizer(self)
            .encode(text, true)
            .map_err(|e| anyhow!("Encode error {}", e))?;

        let token_ids = Tensor::new(tokens.get_ids(), self.device())?.unsqueeze(0)?;
        let attention_mask = Tensor::new(tokens.get_attention_mask(), self.device())?.unsqueeze(0)?;
        let token_type_ids = token_ids.zeros_like()?;
        info!("running inference {:?}", token_ids.shape());
        let embeddings = model.forward(&token_ids, &token_type_ids, Some(&attention_mask))?;
        info!("generated embeddings {:?}", embeddings.shape());
        let embeddings = Self::mean_pooling(&embeddings, &attention_mask)?;
        info!("pooled embeddings {:?} in {:?} s", embeddings.shape(), start.elapsed());
        Ok(embeddings)
    }
}

impl Model<BertModel, Config> for RetriverModel
{
    fn name(&self) -> &ModelName 
    {
        &self.model_name
    }

    fn tokenizer(&self) -> &Tokenizer 
    {
        &self.tokenizer
    }

    fn tokenizer_mut(&mut self) -> &mut Tokenizer 
    {
        &mut self.tokenizer
    }

    fn config(&self) -> &Config 
    {
        &self.config
    }
    fn embedding_config(&self) -> Arc<EmbeddingConfiguration> 
    {
        Arc::clone(&self.embedding_config)
    }

    fn device(&self) -> &Device 
    {
        &self.device
    }
    fn model_is_loaded(&self) -> bool 
    {
        self.model.is_some()
    }
    fn dimension(&self) -> usize 
    {
        self.dimension
    }

    /// Загружает BERT модель в память.
    ///
    /// # Процесс загрузки:
    ///
    /// 1. **Проверка**: Если модель уже загружена, возвращает Ok
    /// 2. **Загрузка весов**: Читает файл с весами модели
    /// 3. **Инициализация BERT**: Создает BertModel с загруженными весами
    /// 4. **Блокирующая операция**: Загрузка выполняется в отдельном потоке (spawn_blocking)
    ///    чтобы не блокировать async runtime
    ///
    /// # Возвращает
    /// * `Ok(())` - Модель успешно загружена или уже была загружена
    /// * `Err` - Если не удалось загрузить файл модели или инициализировать BERT
    ///
    /// # Примечание
    /// Загрузка модели - тяжелая операция (может занять несколько секунд).
    /// Модель загружается на устройство, выбранное при инициализации (CPU/CUDA).
    async fn load_model(&mut self) -> Result<()>
    {
        if self.model.is_some()
        {
            return Ok(())
        }
        let start = std::time::Instant::now();
        let config = self.config().clone();
        let model_name = self.model_name.as_ref().to_owned();
        let device = self.device().clone();
        let emb_cfg = self.embedding_config();

        // Загрузка весов модели из файла (PyTorch формат)
        let vb = VarBuilder::from_pth(&emb_cfg.retriver_model_path, DTYPE, &device)
            .context(format!("Error load retriver model from file {}", emb_cfg.retriver_model_path.display()))?;

        // Загрузка модели в отдельном потоке, так как это блокирующая операция
        let result: Result<BertModel> = tokio::task::spawn_blocking(move ||
        {
            let model = BertModel::load(vb, &config).context("Error load retriver model")?;
            info!("retriver model {} loaded in {:?}", model_name, start.elapsed());
            Ok(model)
        }).await?;

        self.model = Some(result?);
        Ok(())
    }
    fn unload_model(&mut self)
    {
        self.model = None;
    }
    fn model(&self) -> Result<&BertModel> 
    {
        let model = self.model.as_ref()
            .context(format!("Model {} is not initialized!", self.model_name.as_ref()))?;
        Ok(model)
    }
}

#[cfg(test)]
mod tests
{
    use std::sync::Arc;

    use candle_transformers::models::bert::DTYPE;
    use candle_core::Tensor;
    use tokenizers::PaddingParams;
    use tracing::info;

    use crate::model::Model;
    use crate::retriver::{RetriverModel};
    use crate::{EmbeddingConfiguration, logger};
    use anyhow::Result;
    #[tokio::test]
    async fn test_model()
    {
        logger::init();
        let start = std::time::Instant::now();
        let cfg = Arc::new(EmbeddingConfiguration::default());
        let mut model = RetriverModel::new(cfg).await.unwrap();
        let sentences = [
            "Документы (сведения), не содержащие налоговую тайну, используемые налоговыми органами при реализации своих полномочий в отношениях, регулируемых законодательством о налогах и сборах, передаются налоговым органом налогоплательщику - физическому лицу, зарегистрированному в единой системе идентификации и аутентификации, в электронной форме через личный кабинет на едином портале государственных и муниципальных услуг, если такой порядок передачи документов (сведений), не содержащих налоговую тайну, предусмотрен настоящим Кодексом или если указанные документы (сведения) включены в перечень, утверждаемый в соответствии с абзацем первым пункта 9 настоящей статьи",
            "Что происходит со сведениями не содержащими налоговую тайну?",
            "Документы (сведения), не содержащие налоговую тайну, используемые налоговыми органами при реализации своих полномочий в отношениях, регулируемых законодательством о налогах и сборах, налогоплательщикам - физическим лицам, зарегистрированным в единой системе идентификации и аутентификации, направленные через личный кабинет на едином портале государственных и муниципальных услуг, на бумажном носителе по почте не направляются, если иное не предусмотрено пунктом 2 статьи 112 настоящего Кодекса.",
            "The new movie is awesome",
            "The cat plays in the garden",
            "A woman watches TV",
            "The new movie is so great",
            "Do you like pizza?",
        ];
        let n_sentences = sentences.len();
        if let Some(pp) = model.tokenizer_mut().get_padding_mut() 
        {
            pp.strategy = tokenizers::PaddingStrategy::BatchLongest
        } 
        else 
        {
            let pp = PaddingParams 
            {
                strategy: tokenizers::PaddingStrategy::BatchLongest,
                ..Default::default()
            };
            model.tokenizer_mut().with_padding(Some(pp));
        }
        let tokens = model.tokenizer()
            .encode_batch(sentences.to_vec(), true).unwrap();

        let token_ids = tokens
            .iter()
            .map(|tokens| {
                let tokens = tokens.get_ids().to_vec();
                Ok(Tensor::new(tokens.as_slice(), model.device()).unwrap())
            })
            .collect::<Result<Vec<_>>>().unwrap();

        let attention_mask = tokens
            .iter()
            .map(|tokens| {
                let tokens = tokens.get_attention_mask().to_vec();
                Ok(Tensor::new(tokens.as_slice(), model.device()).unwrap())
            })
            .collect::<Result<Vec<_>>>().unwrap();

        let token_ids = Tensor::stack(&token_ids, 0).unwrap();
        let attention_mask = Tensor::stack(&attention_mask, 0).unwrap();
        let token_type_ids = token_ids.zeros_like().unwrap();
        info!("running inference on batch {:?}", token_ids.shape());
        model.load_model().await.unwrap();
        let embeddings = model.model().unwrap().forward(&token_ids, &token_type_ids, Some(&attention_mask)).unwrap();
        info!("generated embeddings {:?}", embeddings.shape());
        let embeddings = mean_pooling(&embeddings, &attention_mask).unwrap();
        //let embeddings = normalize_l2(&embeddings).unwrap();
        info!("pooled embeddings {:?}", embeddings.shape());
        //let vec: Vec<f32> = embeddings.to_vec1().unwrap();
        let mut similarities = vec![];
        for i in 0..n_sentences 
        {
            let e_i = embeddings.get(i).unwrap();
            for j in (i + 1)..n_sentences 
            {
                let e_j = embeddings.get(j).unwrap();
                let sum_ij = (&e_i * &e_j).unwrap().sum_all().unwrap().to_scalar::<f32>().unwrap();
                let sum_i2 = (&e_i * &e_i).unwrap().sum_all().unwrap().to_scalar::<f32>().unwrap();
                let sum_j2 = (&e_j * &e_j).unwrap().sum_all().unwrap().to_scalar::<f32>().unwrap();
                let cosine_similarity = sum_ij / (sum_i2 * sum_j2).sqrt();
                similarities.push((cosine_similarity, i, j))
            }
        }
        similarities.sort_by(|u, v| v.0.total_cmp(&u.0));
        for &(score, i, j) in similarities[..5].iter() 
        {
            info!("score: {score:.2} '{}' '{}'", sentences[i], sentences[j])
        }
        // let token_ids = Tensor::new(&tokens[..], context.device()).unwrap().unsqueeze(0).unwrap();
        // let token_type_ids = token_ids.zeros_like().unwrap();
        // println!("Loaded and encoded {:?}", start.elapsed());
        // for idx in 0..1 {
        //     let start = std::time::Instant::now();
        //     let ys = context.model().unwrap().forward(&token_ids, &token_type_ids, None).unwrap();
        //     if idx == 0 {
        //         println!("{ys}");
        //     }
        //     println!("Took {:?}", start.elapsed());
        // }
        info!("Overall work time {:?}", start.elapsed());
    }

    #[allow(dead_code)]
    pub fn normalize_l2(v: &Tensor) -> Result<Tensor>
    {
        Ok(v.broadcast_div(&v.sqr().unwrap().sum_keepdim(1).unwrap().sqrt().unwrap()).unwrap())
    }

    #[allow(dead_code)]
    fn mean_pooling(embeddings: &Tensor, attention_mask: &Tensor) -> Result<Tensor> 
    {
        let attention_mask_for_pooling = attention_mask.to_dtype(DTYPE)?.unsqueeze(2)?;
        let sum_mask = attention_mask_for_pooling.sum(1)?;
        let embeddings = (embeddings.broadcast_mul(&attention_mask_for_pooling)?).sum(1)?;
        let result = embeddings.broadcast_div(&sum_mask)?;
        let result = result.broadcast_div(&result.sqr()?.sum_keepdim(1)?.sqrt()?)?;
        Ok(result)
    }

     #[tokio::test]
    async fn test_embed()
    {
        logger::init();
        let emb_cfg = Arc::new(EmbeddingConfiguration::default());
        let mut retriver = RetriverModel::new(emb_cfg).await.unwrap();
        retriver.load_model().await.unwrap();
        let text = &["Тестовый текст лалалал"];
        let e =  retriver.embed_tensor_batch(text).await;
        info!("{:?}", e);
    }
    #[tokio::test]
    async fn test_embed2()
    {
        logger::init();
        let emb_cfg = Arc::new(EmbeddingConfiguration::default());
        let retriver = RetriverModel::new(emb_cfg).await.unwrap();
        let text = "Тестовый текст лалалал";
        let e =  retriver.embed_tensor(text).await;
        info!("{:?}", e);
    }
}