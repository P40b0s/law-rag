use std::sync::{Arc, OnceLock};

use crate::{EmbeddingConfiguration, model::{Model, ModelName}};
use anyhow::{Context, Result, anyhow};
use candle_core::{DType, Tensor, Device};
use candle_nn::VarBuilder;
use tokenizers::{PaddingParams, Tokenizer};
use tracing::info;
use candle_transformers::models::{bert::{BertModel, Config, DTYPE}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct RerankResult<P>
{
    pub score: f32,
    pub payload: P,
}

pub struct BgeReranker 
{
    model_name: ModelName,
    device: Device,
    config: Config,
    embedding_config: Arc<EmbeddingConfiguration>,
    tokenizer: Tokenizer,
    model: OnceLock<BertModel>
}

impl Model for BgeReranker
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

    async fn load_model(self) -> Result<Self>
    {
        let start = std::time::Instant::now();
        let config = self.config().clone();
        let model_name = self.model_name.as_ref().to_owned();
        let device = self.device().clone();
        let emb_cfg = self.embedding_config();
        let vb = VarBuilder::from_pth(&emb_cfg.retriver_model_path, DTYPE, &device)
            .context(format!("Error load reranker model from file {}", emb_cfg.retriver_model_path.display()))?;
        let result: Result<BertModel> = tokio::task::spawn_blocking(move ||
        {
            let model = BertModel::load(vb, &config).context("Error load reranker model")?;
            info!("reranker model {} loaded in {:?}", model_name, start.elapsed());
            Ok(model)
        }).await?;
        let _ = self.model.set(result?);
        Ok(self)
    }
    fn model(&self) -> Result<&BertModel> 
    {
        let model = self.model.get().context("Model is not initialized!")?;
        Ok(model)
    }
}

impl BgeReranker 
{
    // pub async fn new(emb_cfg: Arc<EmbeddingConfiguration>) -> Result<Self> 
    // {
    //     let device = Device::cuda_if_available(0).unwrap_or(Device::Cpu);
    //     info!("Инициализация реранкера {} на устройстве {:?}", 
    //           model_name.as_ref(), &device);
        
    //     // Загрузка токенайзера и конфигурации
    //     let current_dir = std::env::current_dir()?;
    //     let tokenizer_path = current_dir.join("reranker").join("tokenizer.json");
    //     let config_path = current_dir.join("reranker").join("config.json");
    //     let model_path = current_dir.join("reranker").join("model.safetensors");
        
    //     let tokenizer_bytes = tokio::fs::read(&tokenizer_path).await?;
    //     let mut tokenizer = Tokenizer::from_bytes(&tokenizer_bytes)?;
        
    //     // Настройка padding для пар запрос-документ
    //     let padding = tokenizers::PaddingParams 
    //     {
    //         strategy: tokenizers::PaddingStrategy::BatchLongest,
    //         direction: tokenizers::PaddingDirection::Right,
    //         pad_id: 0,
    //         pad_type_id: 0,
    //         pad_token: "[PAD]".to_string(),
    //         ..Default::default()
    //     };
    //     tokenizer.with_padding(Some(padding));
        
    //     // Загрузка конфигурации
    //     let config_str = tokio::fs::read_to_string(&config_path).await?;
    //     let config: candle_transformers::models::bert::Config = 
    //         serde_json::from_str(&config_str)?;
        
    //     // Загрузка модели
    //     let vb = VarBuilder::from_pth(&model_path, DType::F32, &device)?;
    //     let model = candle_transformers::models::bert::BertModel::load(vb, &config)?;
        
    //     Ok(Self 
    //     {
    //         model,
    //         tokenizer,
    //         device,
    //     })
    // }

    pub async fn new(emb_config: Arc<EmbeddingConfiguration>) -> Result<Self>
    {
        info!("Try load tokenizer.json");
        let device = Device::cuda_if_available(0).unwrap_or(Device::Cpu);
        info!("Running on device: {:?}", &device);
        let tokenizer_file = &emb_config.reranker_tokenizer_path;
        let tokenizer = tokio::fs::read(&tokenizer_file).await
            .with_context(||format!("Error load tokenizer file from {}", tokenizer_file.display()))?;
        let mut tokenizer = Tokenizer::from_bytes(tokenizer)
            .map_err(|e| anyhow::anyhow!("Error deserialize tokenizer file {}", e))?;
        let pp = PaddingParams 
        {
            strategy: tokenizers::PaddingStrategy::BatchLongest,
            direction: tokenizers::PaddingDirection::Right,
            pad_id: 0,
            pad_type_id: 0,
            pad_token: "[PAD]".to_string(),
            ..Default::default()
        };
        tokenizer.with_padding(Some(pp));
        
        let model_config_file = &emb_config.reranker_config_path;
        let model_config = tokio::fs::read(&model_config_file).await
            .context(format!("Error load config file from {}", model_config_file.display()))?;
        let config = serde_json::from_slice(&model_config).context("Error deserialize config file")?;
        let slf = Self
        {
            model_name: ModelName::RerankerM3,
            config,
            embedding_config: emb_config,
            device,
            tokenizer,
            model: OnceLock::<BertModel>::new()
        };
        let with_loaded_model = slf.load_model().await?;
        Ok(with_loaded_model)
    }

    
    
    pub async fn rerank<P: AsRef<str>, R>(
        &self,
        query: &str,
        candidates: R,
        top_k: usize,
    ) -> Result<Vec<RerankResult<P>>> 
    where R: IntoIterator<Item = P>
            
    {
        let start = std::time::Instant::now();
        let candidate_vec: Vec<P> = candidates.into_iter().collect();
        // Подготовка пар для реранкинга
        let pairs: Vec<(String, String)> = candidate_vec
            .iter()
            .map(|candidate| 
            {
                let query_text = format!("{} {}", query, candidate.as_ref());
                (query_text, candidate.as_ref().to_owned())
            })
            .collect();
        
        // Получение скорингов
        let scores = self.compute_scores_batch(&pairs).await?;
        
        // Формирование результатов
        let mut reranked: Vec<RerankResult<P>> = candidate_vec.into_iter()
            .zip(scores.into_iter())
            .map(|(candidate, score)| RerankResult 
            {
                score,
                payload: candidate,
            })
            .collect();
        
        // Сортировка по новым скорингам
        reranked.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        info!("Реранкинг {} кандидатов завершен за {:?}", 
              reranked.len(), start.elapsed());
        
        // Возврат топ-K результатов
        Ok(reranked.into_iter().take(top_k).collect())
    }
    
    async fn compute_scores_batch(&self, pairs: &[(String, String)]) -> Result<Vec<f32>> 
    {
        let device = &self.device;
        
        // Подготовка текстов для токенизации
        let texts: Vec<String> = pairs
            .iter()
            .map(|(query, document)| format!("{} [SEP] {}", query, document))
            .collect();
        
        let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        
        // Токенизация
        let encodings = self.tokenizer.encode_batch(text_refs, true)
            .map_err(|e| anyhow!("Error {} when encode batches", e))?;
        
        // Подготовка тензоров
        let token_ids: Vec<Tensor> = encodings
            .iter()
            .map(|encoding| 
            {
                Tensor::new(encoding.get_ids(), device).context("Error ids encoding")
            })
            .collect::<Result<Vec<_>>>()?;
            
        let attention_masks: Vec<Tensor> = encodings
            .iter()
            .map(|encoding| {
                Tensor::new(encoding.get_attention_mask(), device).context("Error attension mask encoding")
            })
            .collect::<Result<Vec<_>>>()?;
        
        let token_ids = Tensor::stack(&token_ids, 0)?;
        let attention_mask = Tensor::stack(&attention_masks, 0)?;
        let token_type_ids = token_ids.zeros_like()?;
        
        // Прямой проход
        let output = self.model()?.forward(&token_ids, &token_type_ids, Some(&attention_mask))?;
        
        // Извлечение скорингов (берем embedding первого токена [CLS])
        let scores = output
            .narrow(1, 0, 1).context("Error narrow params")?  // Берем первый токен
            .squeeze(1).context("Error squeeze")?       // Убираем размерность токенов
            .to_vec2::<f32>().context("Error vec ranking")?;
        //TODO проверить правильно ли сделал размерность...
        let first_scrore = scores.first().context("Scores is empty!")?;
        // Нормализация скорингов через sigmoid
        let scores = first_scrore
            .into_iter()
            .map(|score| 1.0 / (1.0 + (-*score).exp())) // Sigmoid
            .collect();
        
        Ok(scores)
    }
}

#[cfg(test)]
mod tests
{
    use std::sync::Arc;

    use tracing::info;

    use crate::{EmbeddingConfiguration, logger};

    #[tokio::test]
    async fn test_reranker()
    {
        logger::init();
        let emb_cfg = Arc::new(EmbeddingConfiguration::default());
        let r = super::BgeReranker::new(emb_cfg).await;
        info!("{:#?}", r.err());
    }
}