use std::sync::{Arc, atomic::AtomicUsize, Mutex};
use anyhow::{Context, anyhow};
use candle_transformers::generation::{LogitsProcessor, Sampling};
use rag_core::EmbeddingConfiguration;
use tokenizers::Tokenizer;
use tracing::{debug, info, warn};
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use crate::{Generator, token_output_stream::TokenOutputStream};
use super::generator_trait::GeneratorSettings;

// Using non-quantized llama for 1B model with safetensors format
type ModelWeights = candle_transformers::models::llama::Llama;
type Config = candle_transformers::models::llama::Config;
type Cache = candle_transformers::models::llama::Cache;

/// Llama 1B model generator (using safetensors format)
pub struct GeneratorLlama1b {
    settings: Arc<EmbeddingConfiguration>,
    model_settings: GeneratorSettings,
    system_prompt: String,
    model: Option<ModelWeights>,
    config: Option<Config>,
    cache: Mutex<Option<Cache>>,
    tokenizer: Option<tokenizers::Tokenizer>,
    device: Device,
    size: AtomicUsize,
}

impl GeneratorLlama1b {
   
    pub fn load(settings: Arc<EmbeddingConfiguration>) -> anyhow::Result<Self>
    {
        let device = Device::cuda_if_available(0).unwrap_or(Device::Cpu);
        info!("Running on device: {:?}", &device);
        let slf = Self
        {
            system_prompt: settings.system_prompt.clone(),
            settings,
            model_settings: GeneratorSettings::default(),
            model: None,
            config: None,
            cache: Mutex::new(None),
            tokenizer: None,
            device,
            size: AtomicUsize::new(0)

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

    fn get_device(&self) -> &Device
    {
        &self.device
    }

    fn reset_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            *cache = None;
        }
    }
    fn get_message<C: ToString + AsRef<str>>(&self, query: &str, context: &[C]) -> String 
    {
        let prompt = format!(
        "<|begin_of_text|>\
        <|start_header_id|>system<|end_header_id|>\n\n\
        {}\nКонтекст:\n{}<|eot_id|>\n\
        <|start_header_id|>user<|end_header_id|>\n\n\
        {}<|eot_id|>\n\
        <|start_header_id|>assistant<|end_header_id|>\n\n",
        &self.get_system_prompt(), query, context.iter().map(|c| c.as_ref()).collect::<Vec<_>>().join("\n"));
        prompt
    }
    fn get_eof(&self) -> &'static str
    {
        "<|eot_id|>"
    }

    fn forward(&mut self, input: &candle_core::Tensor, offset: usize) -> anyhow::Result<candle_core::Tensor>
    {
        if let (Some(model), Some(config)) = (self.model.as_ref(), self.config.as_ref())
        {
            let mut cache_guard = self.cache.lock()
                .map_err(|e| anyhow!("Failed to lock cache: {}", e))?;

            // Initialize cache if needed
            if cache_guard.is_none() {
                *cache_guard = Some(Cache::new(true, candle_core::DType::F32, config, &self.device)?);
            }

            let cache = cache_guard.as_mut().unwrap();
            let logits = model.forward(input, offset, cache)?;
            Ok(logits)
        }
        else
        {
            Err(anyhow::Error::msg("Ошибка, модель или конфигурация не загружена!"))
        }
    }
    fn get_logits_processor(&self) -> LogitsProcessor {
        let settings = &self.model_settings;
        let temperature = settings.temperature;
        let top_k = settings.top_k;
        let top_p = settings.top_p;
        let sampling = if temperature <= 0. {
            Sampling::ArgMax
        } else {
            match (top_k, top_p) {
                (None, None) => Sampling::All { temperature },
                (Some(k), None) => Sampling::TopK { k, temperature },
                (None, Some(p)) => Sampling::TopP { p, temperature },
                (Some(k), Some(p)) => Sampling::TopKThenTopP { k, p, temperature },
            }
        };
        LogitsProcessor::from_sampling(settings.seed, sampling)
    }
    fn get_token_stream(&self) -> TokenOutputStream 
    {
        TokenOutputStream::new(self.tokenizer.as_ref().unwrap().clone())
    }
}

// Implement the Generator trait for GeneratorLlama1b
impl super::generator_trait::Generator for GeneratorLlama1b
{
    
    async fn load_model(&mut self) -> anyhow::Result<()> 
    {
        if self.model.is_some() 
        {
            return Ok(());
        }

        let start = std::time::Instant::now();
        let model_path = self.settings.generator_model_path.clone();
        let config_path = self.settings.generator_config_path.clone();
        let device = self.device.clone();

        // Load and parse config manually since Config doesn't implement Deserialize
        let config_content = tokio::fs::read_to_string(&config_path).await
            .context(format!("Error loading config from {}", config_path.display()))?;
        let config_json: serde_json::Value = serde_json::from_str(&config_content)
            .context("Error parsing config JSON")?;

        // Load model weights from safetensors in blocking thread
        let result: anyhow::Result<(ModelWeights, Config)> = tokio::task::spawn_blocking(move || {
            // Create config from JSON
            let config = Config {
                hidden_size: config_json["hidden_size"].as_u64().unwrap_or(2048) as usize,
                intermediate_size: config_json["intermediate_size"].as_u64().unwrap_or(5632) as usize,
                vocab_size: config_json["vocab_size"].as_u64().unwrap_or(128256) as usize,
                num_hidden_layers: config_json["num_hidden_layers"].as_u64().unwrap_or(16) as usize,
                num_attention_heads: config_json["num_attention_heads"].as_u64().unwrap_or(32) as usize,
                num_key_value_heads: config_json["num_key_value_heads"].as_u64().unwrap_or(8) as usize,
                use_flash_attn: false,
                rms_norm_eps: config_json["rms_norm_eps"].as_f64().unwrap_or(1e-5),
                rope_theta: config_json["rope_theta"].as_f64().unwrap_or(500000.0) as f32,
                bos_token_id: config_json["bos_token_id"].as_u64().map(|v| v as u32),
                eos_token_id: None,
                rope_scaling: None,
                max_position_embeddings: config_json["max_position_embeddings"].as_u64().unwrap_or(131072) as usize,
                tie_word_embeddings: config_json.get("tie_word_embeddings").and_then(|v| v.as_bool()).unwrap_or(true),
            };

            let vb = unsafe {
                VarBuilder::from_mmaped_safetensors(&[model_path.clone()], candle_core::DType::F32, &device)
                    .context(format!("Error loading model from {}", model_path.display()))?
            };

            let llama = ModelWeights::load(vb, &config)
                .context("Error loading Llama model")?;

            info!("Llama 1B model loaded in {:?}", start.elapsed());
            Ok((llama, config))
        }).await?;

        let (model, config) = result?;
        self.model = Some(model);
        self.config = Some(config);

        // Store approximate size
        let metadata = std::fs::metadata(&self.settings.generator_model_path)?;
        self.size.store(metadata.len() as usize, std::sync::atomic::Ordering::Relaxed);

        // Reset cache when loading new model
        self.reset_cache();

        Ok(())
    }

    fn unload_model(&mut self) {
        self.model = None;
        self.config = None;
        self.reset_cache();
    }

    fn model_is_loaded(&self) -> bool {
        self.model.is_some()
    }


    fn get_size_in_bytes(&self) -> usize {
        self.size.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn get_system_prompt(&self) -> &str {
        &self.system_prompt
    }

    fn prompt<'a, C: ToString + AsRef<str>>(&'a mut self, query: &'a str, context: &'a[C], sender: tokio::sync::mpsc::UnboundedSender<String>,)
     -> anyhow::Result<()>
    {
            // Reset cache for new generation
            self.reset_cache();

            // match sender.send("ТЕСТ: Начинаю генерацию".to_string())
            // {
            //     Ok(_) => info!("[PROMPT] Тестовое сообщение отправлено"),
            //     Err(e) => info!("[PROMPT] Ошибка отправки: {}", e),
            // }
            // info!("Сообщение должно быть отправлено!");
            let mut tos = self.get_token_stream();
            let prompt_str = self.get_message(query, context);
            let tokens = tos
                .tokenizer()
                .encode(prompt_str, true)
                .map_err(anyhow::Error::msg)?;
            let model_settings = self.model_settings.clone();
            let tokens = tokens.get_ids();
            let to_sample = model_settings.sample_len.saturating_sub(1);
            let mut all_tokens = vec![];
            let mut logits_processor = self.get_logits_processor();
            let eos_token = *tos.tokenizer().get_vocab(true).get(self.get_eof()).context("Error load eos token")?;

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
            let _ = sender.send("[END]".to_owned());
            Ok(())
    }
}

#[cfg(test)]
mod tests
{
    use std::sync::Arc;
    use rag_core::EmbeddingConfiguration;
    use tracing::{debug, info};
    use crate::{logger};
    use super::super::generator_trait::Generator;

    #[tokio::test]
    async fn test_load_model()
    {
        logger::init();
        let config = Arc::new(EmbeddingConfiguration::default());
        let mut model = super::GeneratorLlama1b::load(config).unwrap();
        model.load_model().await.unwrap();
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
