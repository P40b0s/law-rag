use candle_core::{DType, Device, IndexOp, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use snafu::ResultExt;
use tokenizers::{PaddingParams, Tokenizer};
use tracing::{info, warn};
use std::path::Path;
use std::sync::Arc;

use crate::{error::{Error, Result}, model::LongContextModel};

pub struct LongContextEmbedder 
{
    model_type: LongContextModel,
    tokenizer: Tokenizer,
    device: Device,
    model: Option<BertModel>,
    // Для BGE-M3 можно использовать ColBERT-like подход
    use_colbert: bool,
    max_length: usize,
}

impl LongContextEmbedder 
{
    pub fn new(model_type: LongContextModel) -> Result<Self> 
    {
       
        let device = Device::cuda_if_available(0).unwrap_or(Device::Cpu);
        info!("Using device: {:?}", device);
        // Загружаем токенизатор
        let mut tokenizer = Self::load_tokenizer(model_type)?;
        
        // Настраиваем padding для длинного контекста
        let max_length = model_type.max_tokens();
        let padding = PaddingParams 
        {
            strategy: tokenizers::PaddingStrategy::Fixed(max_length),
            direction: tokenizers::PaddingDirection::Right,
            pad_to_multiple_of: None,
            pad_id: 0,
            pad_type_id: 0,
            pad_token: "[PAD]".to_string(),
        };
        
        tokenizer.with_padding(Some(padding));
        
        Ok(Self 
        {
            model_type,
            tokenizer,
            device,
            model: None,
            use_colbert: true,
            max_length,
        })
    }

    pub fn with_model(mut self) -> Result<Self>
    {
        let model = Self::load_model(self.model_type, &self.device)?;
        self.model = model;
        Ok(self)
    }
    
    fn load_tokenizer(model_type: LongContextModel) -> Result<Tokenizer> 
    {
        let model_name = model_type.model_name();
        
        match model_type 
        {
            LongContextModel::Bge => 
            {
                // BGE-M3 использует специальный токенизатор
                let tokenizer = Tokenizer::from_pretrained(model_name, None)?;
                
                // Для BGE-M3 можем использовать предобученный токенизатор
                Ok(tokenizer)
            },
            LongContextModel::BgeReranker => 
            {
                // BGE-Reranker использует инструкции для запросов и документов
                let mut tokenizer = Tokenizer::from_pretrained(model_name, None)?;
                
                // Добавляем специальные токены для инструкций
                tokenizer.add_special_tokens(&[
                    tokenizers::AddedToken::from("[CLS]", true),
                    tokenizers::AddedToken::from("[SEP]", true),
                ]);
                
                Ok(tokenizer)
            },
            _ => Ok(Tokenizer::from_pretrained(model_name, None)?)
        }
    }

    pub fn get_tokenizer(&self) -> &Tokenizer
    {
        &self.tokenizer
    }
    
    fn load_model(model_type: LongContextModel, device: &Device) -> Result<Option<BertModel>> 
    {
        let model_name = model_type.model_name();
        info!("Loading model: {}", model_name);
        
        // Создаем конфигурацию модели
        let config = Config 
        { 
            vocab_size: 250002,
            hidden_size: 1024,
            num_hidden_layers: 24,
            num_attention_heads: 16,
            intermediate_size: 4096,
            hidden_act: candle_transformers::models::bert::HiddenAct::Gelu,
            hidden_dropout_prob: 0.1f64,
            max_position_embeddings: 8194,
            type_vocab_size: 1,
            initializer_range: 0.02f64,
            layer_norm_eps: 0.00001f64,
            pad_token_id: 1,
            position_embedding_type: candle_transformers::models::bert::PositionEmbeddingType::Absolute,
            use_cache: true,
            classifier_dropout: None,
            model_type: Some("xlm-roberta".to_owned())
        };
        
        // Загружаем веса модели
        let weights_path = format!("{}/model.safetensors", model_name);
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_path], DTYPE, device)
                .map_err(|e| Error::ModelLoadError {
                    model: model_name.to_string(),
                    source: e.into(),
                })?
        };
        
        // Создаем модель BERT
        let model = BertModel::load(vb, &config)
            .map_err(|e| Error::ModelLoadError 
            {
                model: model_name.to_string(),
                source: e.into(),
            })?;
        
        Ok(Some(model))
    }
    
    pub async fn embed_long_text(&self, text: &str) -> Result<Vec<Vec<f32>>> 
    {
        match self.model_type 
        {
            LongContextModel::Bge => self.embed_bge_m3(text).await,
            LongContextModel::BgeReranker => self.embed_bge_reranker(text).await,
            _ => self.embed_general(text).await,
        }
    }
    
    async fn embed_bge_m3(&self, text: &str) -> Result<Vec<Vec<f32>>> 
    {
        // BGE-M3 требует инструкций для разных типов текста
        // Для обычных текстов используем инструкцию для документов
        let instruction = if text.contains('?') || text.contains("как") || text.contains("почему") {
            "Represent this question for searching relevant passages: "
        } 
        else 
        {
            "Represent this document for retrieval: "
        };
        
        let text_with_instruction = format!("{}{}", instruction, text);
        
        // Токенизация
        let encoding = self.tokenizer
            .encode(text_with_instruction, true)
            .map_err(|e| Error::TokenizationError 
            {
                text: text.to_string(),
                source: e.into(),
            })?;
        
        let input_ids = encoding.get_ids();
        let tt_ids = encoding.get_type_ids();
        let attention_mask = encoding.get_attention_mask();
        
        // Подготавливаем тензоры
        let input_ids_tensor = Tensor::new(input_ids, &self.device)
            .map_err(|e| Error::TensorError { source: e })?;
        let tt_ids_tensor = Tensor::new(tt_ids, &self.device)
            .map_err(|e| Error::TensorError { source: e })?;
        let attention_mask_tensor = Tensor::new(attention_mask, &self.device)
            .map_err(|e| Error::TensorError { source: e })?;
        
        // Проверяем длину и обрезаем если нужно
        let input_ids_tensor = if input_ids.len() > self.max_length 
        {
            input_ids_tensor.narrow(0, 0, self.max_length)
                .map_err(|e| Error::TensorError { source: e })?
        } 
        else 
        {
            input_ids_tensor
        };

        let tt_ids_tensor = if tt_ids.len() > self.max_length 
        {
            tt_ids_tensor.narrow(0, 0, self.max_length)
                .map_err(|e| Error::TensorError { source: e })?
        } 
        else 
        {
            tt_ids_tensor
        };



        
        let attention_mask_tensor = if attention_mask.len() > self.max_length {
            attention_mask_tensor.narrow(0, 0, self.max_length)
                .map_err(|e| Error::TensorError { source: e })?
        } else {
            attention_mask_tensor
        };
        
        // Получаем эмбеддинги от модели
        let embeddings = self.get_model_embeddings(input_ids_tensor, tt_ids_tensor, attention_mask_tensor).await?;
        
        // Нормализуем эмбеддинги (как делается в BGE)
        let normalized_embeddings = self.normalize_embeddings(embeddings);
        
        Ok(vec![normalized_embeddings])
    }
    
    async fn embed_bge_reranker(&self, text: &str) -> Result<Vec<Vec<f32>>> {
        // BGE-Reranker может использовать специальные токены
        let formatted_text = if !text.contains("[CLS]") {
            format!("[CLS]{}[SEP]", text)
        } else {
            text.to_string()
        };
        
        // Токенизация
        let encoding = self.tokenizer
            .encode(formatted_text, true)
            .map_err(|e| Error::TokenizationError {
                text: text.to_string(),
                source: e.into(),
            })?;
        
        let input_ids = encoding.get_ids();
        let tt_ids = encoding.get_type_ids();
        let attention_mask = encoding.get_attention_mask();
        
        // Подготавливаем тензоры
        let input_ids_tensor = Tensor::new(input_ids, &self.device)
            .map_err(|e| Error::TensorError { source: e })?;
         let tt_ids_tensor = Tensor::new(tt_ids, &self.device)
            .map_err(|e| Error::TensorError { source: e })?;
        let attention_mask_tensor = Tensor::new(attention_mask, &self.device)
            .map_err(|e| Error::TensorError { source: e })?;
        
        // Получаем эмбеддинги
        let embeddings = self.get_model_embeddings(input_ids_tensor, tt_ids_tensor, attention_mask_tensor).await?;
        
        Ok(vec![embeddings])
    }
    
    async fn embed_general(&self, text: &str) -> Result<Vec<Vec<f32>>> {
        // Общая реализация для других моделей
        let encoding = self.tokenizer
            .encode(text, true)
            .map_err(|e| Error::TokenizationError {
                text: text.to_string(),
                source: e.into(),
            })?;
        
        let input_ids = encoding.get_ids();
        let tt_ids = encoding.get_type_ids();
        let attention_mask = encoding.get_attention_mask();
        
        // Подготавливаем тензоры
        let input_ids_tensor = Tensor::new(input_ids, &self.device)
            .map_err(|e| Error::TensorError { source: e })?;

        let tt_ids_tensor = Tensor::new(tt_ids, &self.device)
            .map_err(|e| Error::TensorError { source: e })?;

        let attention_mask_tensor = Tensor::new(attention_mask, &self.device)
            .map_err(|e| Error::TensorError { source: e })?;
        
        // Получаем эмбеддинги
        let embeddings = self.get_model_embeddings(input_ids_tensor, tt_ids_tensor,  attention_mask_tensor).await?;
        
        Ok(vec![embeddings])
    }
    
    async fn get_model_embeddings(
        &self,
        input_ids: Tensor,
        tt_ids: Tensor,
        attention_mask: Tensor,
    ) -> Result<Vec<f32>> {
        // Используем асинхронный контекст для выполнения инференса
        let model = self.model.as_ref()
            .ok_or_else(|| Error::ModelNotLoaded {
                model_name: self.model_type.model_name().to_string(),
            })?;
        
        // Добавляем размерность батча
        let input_ids = input_ids.unsqueeze(0)
            .map_err(|e| Error::TensorError { source: e })?;
        let attention_mask = attention_mask.unsqueeze(0)
            .map_err(|e| Error::TensorError { source: e })?;
        
        // Получаем выход модели
        let hidden_states = model
            .forward(&input_ids, &tt_ids, Some(&attention_mask))
            .map_err(|e| Error::InferenceError { source: e })?;
        
        // Используем embedding токена [CLS] для получения векторного представления
        // Берем скрытые состояния последнего слоя
        let el_count = hidden_states.elem_count();
        let last_hidden_state = hidden_states.i(el_count -1)?;
        
        // Извлекаем эмбеддинг токена [CLS] (первый токен)
        let cls_embedding = last_hidden_state
            .narrow(1, 0, 1)?
            .squeeze(1)?;
        
        // Преобразуем в вектор f32
        let embedding_vec: Vec<f32> = cls_embedding
            .to_vec1()
            .map_err(|e| Error::TensorError { source: e })?;
        
        Ok(embedding_vec)
    }
    
    fn normalize_embeddings(&self, embeddings: Vec<f32>) -> Vec<f32> {
        // Нормализуем вектор (L2 нормализация)
        let norm = embeddings.iter()
            .map(|&x| x * x)
            .sum::<f32>()
            .sqrt();
        
        if norm > 0.0 {
            embeddings.iter()
                .map(|&x| x / norm)
                .collect()
        } else {
            embeddings
        }
    }
    
    pub async fn embed_batch_long(
        &self, 
        texts: &[String]
    ) -> Result<Vec<Vec<f32>>> {
        let mut all_embeddings = Vec::new();
        
        // Для батчинга можно оптимизировать, но пока делаем последовательно
        for text in texts.iter() {
            let embeddings = self.embed_long_text(text).await?;
            all_embeddings.extend(embeddings);
        }
        
        Ok(all_embeddings)
    }
    
    // Метод для работы с очень длинными текстами
    pub async fn embed_very_long_text(
        &self,
        text: &str,
        chunk_size: Option<usize>,
        overlap: Option<usize>,
    ) -> Result<Vec<Vec<f32>>> {
        let chunk_size = chunk_size.unwrap_or(512);
        let overlap = overlap.unwrap_or(50);
        
        // Разбиваем текст на чанки
        let chunks = self.chunk_text(text, chunk_size, overlap)?;
        
        // Эмбеддим каждый чанк
        let mut chunk_embeddings = Vec::new();
        for chunk in chunks {
            let embeddings = self.embed_long_text(&chunk).await?;
            chunk_embeddings.extend(embeddings);
        }
        
        // Для получения общего эмбеддинга длинного текста можно усреднить эмбеддинги чанков
        if !chunk_embeddings.is_empty() {
            let avg_embedding = self.average_embeddings(&chunk_embeddings);
            Ok(vec![avg_embedding])
        } else {
            Ok(vec![])
        }
    }
    
    fn chunk_text(
        &self,
        text: &str,
        chunk_size: usize,
        overlap: usize,
    ) -> Result<Vec<String>> {
        // Простая реализация разбивки текста на чанки
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut chunks = Vec::new();
        let mut start = 0;
        
        while start < words.len() {
            let end = (start + chunk_size).min(words.len());
            let chunk = words[start..end].join(" ");
            chunks.push(chunk);
            
            if end == words.len() {
                break;
            }
            
            start = end.saturating_sub(overlap);
        }
        
        Ok(chunks)
    }
    
    fn average_embeddings(&self, embeddings: &[Vec<f32>]) -> Vec<f32> {
        if embeddings.is_empty() {
            return Vec::new();
        }
        
        let dimension = embeddings[0].len();
        let mut sum = vec![0.0; dimension];
        
        for embedding in embeddings {
            for (i, &value) in embedding.iter().enumerate() {
                sum[i] += value;
            }
        }
        
        let count = embeddings.len() as f32;
        sum.iter().map(|&x| x / count).collect()
    }
}