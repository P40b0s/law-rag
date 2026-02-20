use std::sync::{Arc, OnceLock};

use crate::{model::{Model}};
use anyhow::{Context, Result, anyhow};
use candle_core::{DType, Device, IndexOp, Tensor};
use candle_nn::VarBuilder;
use rag_core::{EmbeddingConfiguration, LocalModelConfig, ModelName, RerankResult, Reranker};
use tokenizers::{PaddingParams, Tokenizer};
use tracing::info;
use candle_transformers::models::xlm_roberta::{XLMRobertaForSequenceClassification, Config};
use serde::{Deserialize, Serialize};
const DTYPE: DType = DType::F32;

/// BGE Reranker - модель для переранжирования результатов поиска.
///
/// Реранкер оценивает релевантность пар (query, document) и присваивает им скоры.
/// Используется для улучшения качества поиска после первичного этапа retrieval.
///
/// # Архитектура
/// - Базовая модель: XLM-RoBERTa (BAAI/bge-reranker-v2-m3)
/// - Входной формат: `[query] [SEP] [document]`
/// - Прямой проход через XLM-RoBERTa sequence classification
/// - Модель возвращает логит релевантности
/// - Нормализация через sigmoid для получения вероятности релевантности [0, 1]
///
/// # Использование
/// 1. Первичный поиск с помощью RetriverModel (получение топ-N кандидатов)
/// 2. Реранкинг с помощью BgeReranker (уточнение порядка и отбор топ-K)
pub struct BgeReranker
{
    device: Device,
    internal_model_config: Config,
    embedding_config: Arc<EmbeddingConfiguration>,
    local_model_config: LocalModelConfig,
    tokenizer: Tokenizer,
    model: Option<XLMRobertaForSequenceClassification>
}

impl Model for BgeReranker
{
    fn name(&self) -> &ModelName 
    {
        &self.local_model_config.name
    }

    fn tokenizer(&self) -> &Tokenizer 
    {
        &self.tokenizer
    }

    fn tokenizer_mut(&mut self) -> &mut Tokenizer 
    {
        &mut self.tokenizer
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
        self.local_model_config.dimension
        .or(Some(self.internal_model_config.hidden_size))
        .unwrap_or(1)
    }

    async fn load_model(&mut self) -> Result<()>
    {
        if self.model.is_none()
        {
            let start = std::time::Instant::now();
            let config = self.internal_model_config.clone();
            let model_name = self.local_model_config.name.as_ref().to_owned();
            let device = self.device().clone();

            // Чтение файла safetensors
            let model_path = self.local_model_config.model_path.clone();
            let model_data = tokio::fs::read(&model_path).await
                .context(format!("Error reading reranker model file {}", model_path.display()))?;

            let result: Result<XLMRobertaForSequenceClassification> = tokio::task::spawn_blocking(move ||
            {
                // Загрузка из safetensors формата
                let vb = VarBuilder::from_buffered_safetensors(model_data, DTYPE, &device)
                    .context("Error creating VarBuilder from safetensors")?;
                // XLMRobertaForSequenceClassification для реранкинга использует 1 label (score)
                let model = XLMRobertaForSequenceClassification::new(1, &config, vb)
                    .context("Error load reranker model")?;
                info!("reranker model {} loaded in {:?}", model_name, start.elapsed());
                Ok(model)
            }).await?;
            self.model = Some(result?);
            Ok(())
        }
        else
        {
            Ok(())
        }
    }
    fn unload_model(&mut self)
    {
        self.model = None;
    }
    fn forward(
            &self,
            input_ids: &Tensor,
            attention_mask: &Tensor,
            token_type_ids: &Tensor,
        ) -> Result<Tensor> 
    {
        if let Some(model) = self.model.as_ref()
        {
            let tensor = model.forward(input_ids, attention_mask, token_type_ids)?;
            Ok(tensor)
        }
        else
        {
            Err(anyhow!("Model {} is not initialized!", self.local_model_config.name.as_ref()))
        }
    }
}

impl BgeReranker 
{
    pub async fn new(emb_config: Arc<EmbeddingConfiguration>) -> Result<Self>
    {
        let model_name = ModelName::BgeRerankerV2M3;
        let local_config = emb_config
            .find_reranker(&model_name)?;
        info!("Try load {}", local_config.tokenizer_path.display());
        let device = Device::cuda_if_available(0).unwrap_or(Device::Cpu);
        info!("Running on device: {:?}", &device);
        let tokenizer = tokio::fs::read(&local_config.tokenizer_path).await
            .with_context(|| format!("Error load tokenizer file from {}", local_config.tokenizer_path.display()))?;
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

        let model_config = tokio::fs::read(&local_config.config_path).await
            .context(format!("Error load config file from {}", local_config.config_path.display()))?;
        let internal_model_config = serde_json::from_slice(&model_config).context("Error deserialize config file")?;
        let slf = Self
        {
            internal_model_config,
            local_model_config: local_config.to_owned(),
            embedding_config: emb_config,
            device,
            tokenizer,
            model: None
        };
        Ok(slf)
    }

    /// Переранжирует кандидатов на основе их релевантности к запросу.
    ///
    /// # Процесс реранкинга:
    ///
    /// 1. **Подготовка пар**: Каждый кандидат объединяется с запросом
    /// 2. **Вычисление скоров**: Модель оценивает релевантность каждой пары
    /// 3. **Сортировка**: Кандидаты сортируются по убыванию скора
    /// 4. **Отбор топ-K**: Возвращаются только K наиболее релевантных результатов
    ///
    /// # Аргументы
    /// * `query` - Поисковый запрос
    /// * `candidates` - Итератор кандидатов для переранжирования
    /// * `top_k` - Количество лучших результатов для возврата
    ///
    /// # Возвращает
    /// Вектор `RerankResult` с отсортированными по релевантности результатами (топ-K)
    ///
    /// # Пример
    /// ```ignore
    /// let results = reranker.rerank(
    ///     "налоговая декларация",
    ///     search_results,
    ///     5  // Вернуть топ-5 результатов
    /// ).await?;
    /// ```
    pub async fn rerank<P: AsRef<str> + ToString, R>(
        &self,
        query: &str,
        candidates: R,
        top_k: usize,
    ) -> Result<Vec<RerankResult<P>>>
    where R: IntoIterator<Item = P>
            
    {
        let start = std::time::Instant::now();
        let candidate_vec: Vec<P> = candidates.into_iter().collect();
        // Подготовка пар для реранкинга (query, document)
        let pairs: Vec<(String, String)> = candidate_vec
            .iter()
            .map(|candidate|
            {
                (query.to_string(), candidate.as_ref().to_owned())
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
                db_object: candidate,
            })
            .collect();
        
        // Сортировка по новым скорингам
        reranked.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        info!("Реранкинг {} кандидатов завершен за {:?}", 
              reranked.len(), start.elapsed());
        
        let min_score = self.embedding_config.min_reranking_score;
        // Возврат топ-K результатов с фильтрацией по минимальному скору
        Ok(reranked.into_iter()
            .filter(|r| r.score >= min_score)
            .take(top_k)
            .collect())
    }
    
    /// Вычисляет скоры релевантности для батча пар (query, document).
    ///
    /// # Процесс вычисления:
    ///
    /// 1. **Форматирование**: Объединение пар в формат `[query] [SEP] [document]`
    /// 2. **Токенизация**: Преобразование текстов в токены с padding
    /// 3. **Создание тензоров**: token_ids, attention_mask, token_type_ids
    /// 4. **XLM-RoBERTa forward pass**: Классификация релевантности
    /// 5. **Извлечение логитов**: Модель возвращает логиты [batch_size, 1]
    /// 6. **Нормализация**: Sigmoid для преобразования в вероятности [0, 1]
    ///
    /// # Аргументы
    /// * `pairs` - Слайс пар (query, document) для оценки
    ///
    /// # Возвращает
    /// Вектор нормализованных скоров релевантности (по одному на пару)
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
        
        // Прямой проход через XLM-RoBERTa classifier
        // output shape: [batch_size, num_labels] (для реранкинга num_labels=1)
        let logits = self.forward(&token_ids, &attention_mask, &token_type_ids)?;

        // Извлечение скоров (логитов) из тензора
        // Для sequence classification с 1 label: [batch_size, 1] -> [batch_size]
        let scores = logits
            .squeeze(1)?  // [batch_size, 1] -> [batch_size]
            .to_vec1::<f32>()  // Преобразуем в Vec<f32>
            .context("Error converting scores to vector")?;

        // Нормализация скорингов через sigmoid: σ(x) = 1 / (1 + e^(-x))
        // Преобразует скор в вероятность релевантности в диапазоне [0, 1]
        let normalized_scores: Vec<f32> = scores
            .iter()
            .map(|&score| 1.0 / (1.0 + (-score).exp()))
            .collect();

        Ok(normalized_scores)
    }
}


impl Reranker for BgeReranker
{
    fn rerank<P: AsRef<str> + ToString + Send + Sync, R: Send + Sync>(
            &self,
            query: &str,
            candidates: R,
            top_k: usize,
        ) -> impl std::future::Future<Output = anyhow::Result<Vec<RerankResult<P>>>> + Send
        where R: IntoIterator<Item = P> 
    {
        async move
        {

            let start = std::time::Instant::now();
            let candidate_vec: Vec<P> = candidates.into_iter().collect();
            // Подготовка пар для реранкинга (query, document)
            let pairs: Vec<(String, String)> = candidate_vec
                .iter()
                .map(|candidate|
                {
                    (query.to_string(), candidate.as_ref().to_owned())
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
                    db_object: candidate,
                })
                .collect();
            
            // Сортировка по новым скорингам
            reranked.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
            
            info!("Реранкинг {} кандидатов завершен за {:?}", 
                reranked.len(), start.elapsed());
            
            let min_score = self.embedding_config.min_reranking_score;
            // Возврат топ-K результатов с фильтрацией по минимальному скору
            Ok(reranked.into_iter()
                .filter(|r| r.score >= min_score)
                .take(top_k)
                .collect())
            }
    }
}

#[cfg(test)]
mod tests
{
    use std::sync::Arc;

    use rag_core::EmbeddingConfiguration;
    use tracing::info;

    use crate::{logger};

    #[tokio::test]
    async fn test_reranker()
    {
        logger::init();
        let emb_cfg = Arc::new(EmbeddingConfiguration::default());
        let r = super::BgeReranker::new(emb_cfg).await;
        info!("{:#?}", r.err());
    }
}