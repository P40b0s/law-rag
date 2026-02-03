use std::{path::Path, sync::Arc};
use anyhow::{Context, anyhow};
use candle_transformers::generation::{LogitsProcessor, Sampling};
use futures::future::BoxFuture;
use tokenizers::Tokenizer;
use std::fs::File;
use tracing::{info, warn};
use candle_core::{Device, Tensor, quantized::gguf_file::{self, Content}};
use crate::{EmbeddingConfiguration, token_output_stream::TokenOutputStream};

type ModelWeights = candle_transformers::models::quantized_llama::ModelWeights;
#[derive(Debug, Clone)]
pub struct GeneratorSettings
{
    /// The length of the sample to generate (in tokens).
    sample_len: usize,

    /// The temperature used to generate samples, use 0 for greedy sampling.
    temperature: f64,

    /// Nucleus sampling probability cutoff.
    top_p: Option<f64>,

    /// Only sample among the top K samples.
    top_k: Option<usize>,

    /// The seed to use when generating random samples.
    seed: u64,

    /// Process prompt elements separately.
    split_prompt: bool,

    /// Run on CPU rather than GPU even if a GPU is available.
    cpu: bool,

    /// Penalty to be applied for repeating tokens, 1. means no penalty.
    repeat_penalty: f32,

    /// The context size to consider for the repeat penalty.
    repeat_last_n: usize,
    n_ctx: u32
}

impl Default for GeneratorSettings
{
    fn default() -> Self 
    {
        GeneratorSettings
        {
            sample_len: 1000,
            temperature: 0.6,
            top_p: Some(0.9),
            top_k: Some(30),
            seed: 299792458,
            split_prompt: false,
            cpu: true,
            repeat_penalty: 1.1,
            repeat_last_n: 64,
            n_ctx: 8192
        }
    }
}

pub struct Generator
{
    settings: Arc<EmbeddingConfiguration>,
    model_settings: GeneratorSettings,
    system_prompt: String,
    model: Option<ModelWeights>,
    tokenizer: Option<tokenizers::Tokenizer>,
    device: Device
}

impl Generator
{
    pub fn load(settings: Arc<EmbeddingConfiguration>) -> anyhow::Result<Self>
    {
        let device =  Device::cuda_if_available(0).unwrap_or(Device::Cpu);
        info!("Running on device: {:?}", &device);
        let slf = Self 
        { 
            settings,
            model_settings: GeneratorSettings::default(),
            system_prompt: "Ты - ассистент RAG системы, ты должен отвечать на вопросы пользователей используя ТОЛЬКО предоставленный контекст для формирования ответа. начало ответа должно звучать так: `На основе имеющейся у меня информации: {далее идет твой ответ}`.  Если в контексте нет информации, скажи: \"Не могу ответить на основе имеющейся информации\"".to_owned(),
            model: None,
            tokenizer: None,
            device
        };
        let slf = slf.load_tokenizer()?;
        Ok(slf)
    }
    fn format_size(size_in_bytes: usize) -> String 
    {
        if size_in_bytes < 1_000 
        {
            format!("{size_in_bytes}B")
        } 
        else if size_in_bytes < 1_000_000 
        {
            format!("{:.2}KB", size_in_bytes as f64 / 1e3)
        } 
        else if size_in_bytes < 1_000_000_000 
        {
            format!("{:.2}MB", size_in_bytes as f64 / 1e6)
        } 
        else 
        {
            format!("{:.2}GB", size_in_bytes as f64 / 1e9)
        }
    }
    async fn load_tensors(&self) -> anyhow::Result<(Content, File)>
    {
        let start = std::time::Instant::now();
        let path = self.settings.generator_model_path.clone();
        let model = tokio::task::spawn_blocking(move ||
        {
            let mut file = std::fs::File::open(&path)?;
            let content = 
            {
                let model = gguf_file::Content::read(&mut file).map_err(|e| e.with_path(path))?;
                let mut total_size_in_bytes = 0;
                for (_, tensor) in model.tensor_infos.iter() 
                {
                    let elem_count = tensor.shape.elem_count();
                    total_size_in_bytes +=
                        elem_count * tensor.ggml_dtype.type_size() / tensor.ggml_dtype.block_size();
                }
                info!(
                    "loaded {:?} tensors ({}) in {:.2}s",
                    model.tensor_infos.len(),
                    Self::format_size(total_size_in_bytes),
                    start.elapsed().as_secs_f32(),
                );
                model
            };
            Ok((content, file))
        }).await?;
        model
    }
    fn get_device(&self) -> &Device
    {
        &self.device
    }
    pub async fn load_model(mut self) -> anyhow::Result<Self> 
    {
        let (content, mut file) = self.load_tensors().await?;
        let device = self.get_device();
        let model = ModelWeights::from_gguf(content,  &mut file, device)?;
        info!("model built");
        self.model = Some(model);
        Ok(self)
    }
    pub fn unload_model(&mut self)
    {
        self.model = None;
    }
    fn load_tokenizer(mut self) -> anyhow::Result<Self> 
    {
        let path = &self.settings.generator_tokenizer_path;
        if path.exists()
        {
            let tokenizer = Tokenizer::from_file(path).map_err(|e| anyhow!("Error {e}, when loading tokenizer on path: `{}`", path.display()))?;
            self.tokenizer = Some(tokenizer);
            Ok(self)
        }
        else 
        {
            Err(anyhow::Error::msg(["Файл токенов не найден ->", &path.display().to_string()].concat()))
        }
    }

    fn get_settings(&self) -> &GeneratorSettings 
    {
        &self.model_settings
    }
    ///generate message for model with query and context
    fn get_message_gemma(&self, query: &str, context: &[&str]) -> String 
    {
        let prompt = format!("<|im_start|>system\n{}\nКонтекст:\n{}<|im_end|>
        <|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
        &self.system_prompt, query, context.join("\n"));
        prompt
    }
    fn get_eof_gemma(&self) -> &'static str
    {
        "<|im_end|>"
    }

    fn get_message_llama<C: ToString>(&self, query: &str, context: &[C]) -> String 
    {
        let prompt = format!(
        "<|begin_of_text|>\
        <|start_header_id|>system<|end_header_id|>\n\n\
        {}\nКонтекст:\n{}<|eot_id|>\n\
        <|start_header_id|>user<|end_header_id|>\n\n\
        {}<|eot_id|>\n\
        <|start_header_id|>assistant<|end_header_id|>\n\n",
        &self.system_prompt, query, context.iter().map(|c| c.to_string()).collect::<Vec<_>>().join("\n"));
        prompt
    }
    fn get_eof_llama(&self) -> &'static str
    {
        "<|eot_id|>"
    }
    fn forward(&mut self, input: &candle_core::Tensor, offset: usize) -> anyhow::Result<candle_core::Tensor> 
    {
        if let Some(model) = self.model.as_mut()
        {
            let forw = model.forward(input, offset)?;
            Ok(forw)
        }
        else 
        {
            Err(anyhow::Error::msg("Ошибка, модель не загружена!"))
        }
    }

    pub fn get_token_stream(&self) -> TokenOutputStream
    {
        TokenOutputStream::new(self.tokenizer.as_ref().unwrap().clone())
    }
    pub fn get_logits_processor(&self) -> LogitsProcessor
    {
        let logits_processor = 
        {
            let settings = &self.model_settings;
            let temperature = settings.temperature;
            let top_k = settings.top_k;
            let top_p = settings.top_p;
            let sampling = if temperature <= 0. 
            {
                Sampling::ArgMax
            } 
            else 
            {
                match (top_k, top_p) 
                {
                    (None, None) => Sampling::All { temperature },
                    (Some(k), None) => Sampling::TopK { k, temperature },
                    (None, Some(p)) => Sampling::TopP { p, temperature },
                    (Some(k), Some(p)) => Sampling::TopKThenTopP { k, p, temperature },
                }
            };
            LogitsProcessor::from_sampling(settings.seed, sampling)
        };
        logits_processor
    }
}

pub trait ModelPrompt 
{
     fn prompt<'a, C: ToString>(
        &'a mut self,
        query: &'a str,
        context: &'a[C],
        sender: tokio::sync::mpsc::UnboundedSender<String>,
    ) -> anyhow::Result<()>;
}

impl ModelPrompt for Generator
{
    fn prompt<'a, C: ToString>(&'a mut self, query: &'a str, context: &'a[C], sender: tokio::sync::mpsc::UnboundedSender<String>,)
     -> anyhow::Result<()>
    {
            match sender.send("ТЕСТ: Начинаю генерацию".to_string()) 
            {
                Ok(_) => info!("[PROMPT] Тестовое сообщение отправлено"),
                Err(e) => info!("[PROMPT] Ошибка отправки: {}", e),
            }
            info!("Сообщение должно быть отправлено!");
            let mut tos = self.get_token_stream();
            let prompt_str = self.get_message_llama(query, context);
            let tokens = tos
                .tokenizer()
                .encode(prompt_str, true)
                .map_err(anyhow::Error::msg)?;
            let model_settings = self.model_settings.clone();
            let tokens = tokens.get_ids();
            let to_sample = model_settings.sample_len.saturating_sub(1);
            let mut all_tokens = vec![];
            let mut logits_processor = self.get_logits_processor();
            let eos_token = *tos.tokenizer().get_vocab(true).get(self.get_eof_llama()).context("Error load eos token")?;

            let start_prompt_processing = std::time::Instant::now();
            let mut next_token = if !model_settings.split_prompt 
            {
                let input = Tensor::new(tokens, &self.device)?.unsqueeze(0)?;
                let logits = self.forward(&input, 0)?;
                let logits = logits.squeeze(0)?;
                logits_processor.sample(&logits)?
            } 
            else 
            {
                let mut next_token = 0;
                for (pos, token) in tokens.iter().enumerate() 
                {
                    let input = Tensor::new(&[*token], &&self.device)?.unsqueeze(0)?;
                    let logits = self.forward(&input, pos)?;
                    let logits = logits.squeeze(0)?;
                    next_token = logits_processor.sample(&logits)?
                }
                next_token
            };

            let prompt_dt = start_prompt_processing.elapsed();

            all_tokens.push(next_token);

        
            if let Some(t) = tos.next_token(next_token)? 
            {
                sender.send(t).map_err(|e| anyhow::anyhow!("Ошибка отправки токена!: {}", e))?;
            }

            let start_post_prompt = std::time::Instant::now();

            let mut sampled = 0;
            for index in 0..to_sample 
            {
                let input = Tensor::new(&[next_token],  &self.device)?.unsqueeze(0)?;
                let logits = self.forward(&input, tokens.len() + index)?;
                let logits = logits.squeeze(0)?;
                let logits = if model_settings.repeat_penalty == 1. 
                {
                    logits
                } 
                else 
                {
                    let start_at = all_tokens.len().saturating_sub(model_settings.repeat_last_n);
                    candle_transformers::utils::apply_repeat_penalty(
                        &logits,
                        model_settings.repeat_penalty,
                        &all_tokens[start_at..],
                    )?
                };
                next_token = logits_processor.sample(&logits)?;
                all_tokens.push(next_token);
                if let Some(t) = tos.next_token(next_token)? 
                {
                    if !sender.is_closed()
                    {

                        sender.send(t).context("Ошибка отправки токена в канал")?;
                    }
                    else
                    {
                        warn!("Операция отменена юзером");
                        return Ok(());
                    }
                }
                sampled += 1;
                if next_token == eos_token 
                {
                    break;
                };
            }
            if let Some(rest) = tos.decode_rest().map_err(candle_core::Error::msg)? 
            {
                sender.send(rest).map_err(|e| anyhow::anyhow!("Ошибка отправки оставшегося текста!: {}", e))?;
            }
            let dt = start_post_prompt.elapsed();
            info!(
                "\n\n{:4} prompt tokens processed: {:.2} token/s",
                tokens.len(),
                tokens.len() as f64 / prompt_dt.as_secs_f64(),
            );
            info!(
                "{sampled:4} tokens generated: {:.2} token/s",
                sampled as f64 / dt.as_secs_f64(),
            );
            Ok(())
    }
}

#[cfg(test)]
mod tests
{
    use std::{sync::Arc};

    use tracing::{debug, info};

    use crate::{EmbeddingConfiguration, generator::{ModelPrompt}, logger};

    #[tokio::test]
    async fn test_load_model()
    {
        logger::init();
        let config = Arc::new(EmbeddingConfiguration::default());
        let mut model = super::Generator::load(config).unwrap().load_model().await.unwrap();
        let query = "Какого цвета лицо начальника когда он сердится или не трезв?";
        let context = vec!["Обычно лицо начальника серого цвета", "Когда он выпьет лицо красного цвета", "когда он нервничает у него лицо становиться синим", "Когда сердится то становиться похож на снегиря"];
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        //сообщения из канала не выводятся в тесте!
        tokio::spawn(
            async move 
            {
                while let Some(output) = receiver.recv().await
                {
                    info!("{}", output);
                }
            }
        );
        tokio::task::spawn_blocking(move ||
        {
            let result = model.prompt(query, context.as_slice(), sender);
            debug!("{:?}", result);
        });
    }
}
