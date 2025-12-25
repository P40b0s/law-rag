use std::{ops::Deref, sync::{LazyLock, OnceLock}};
use crate::error::{Error, Result};
use candle_core::Device;
use candle_nn::VarBuilder;
use candle_transformers::models::{bert::{BertModel, Config, DTYPE}};
use serde::{Deserialize, Serialize};
use tokenizers::{PaddingParams, Tokenizer};
use tracing::{error, info};

static MODEL: OnceLock<BertModel> = OnceLock::<BertModel>::new();

pub struct ContextModel
{
    //Каждый чанк будет представлен вектором из `1024` чисел
    pub dimension: usize,
    //чанкуем по `8192` токена
    pub max_tokens: usize,
    pub overlap_tokens: usize,
    pub model_name: ModelName,
    device: Device,
    config: Config,
    tokenizer: Tokenizer
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ModelName
{
    M3
}
impl Deref for ModelName
{
    type Target = str;
    fn deref(&self) -> &Self::Target 
    {
        match self
        {
            ModelName::M3 => "BAAI/bge-m3"
        }
    }
}
impl AsRef<str> for ModelName
{
    fn as_ref(&self) -> &str 
    {
        &self
    }
}

impl ContextModel 
{
    pub async fn new(model_name: ModelName) -> Result<Self>
    {
        info!("Попытка загрузить токенайзер");
        let device = Device::cuda_if_available(0).unwrap_or(Device::Cpu);
        println!("Device: {:?}", &device);
        let max_tokens = 8192;
        let dimension = 1024;
        let tokenizer = tokio::fs::read("./model/tokenizer.json").await?;
        let mut tokenizer = Tokenizer::from_bytes(tokenizer)?;
        let pp = PaddingParams 
        {
            strategy: tokenizers::PaddingStrategy::BatchLongest,
            ..Default::default()
        };
        tokenizer.with_padding(Some(pp));
        let config = tokio::fs::read_to_string("./model/config.json").await?;
        let config = serde_json::from_str(&config)?;
        // let padding = PaddingParams 
        // {
        //     strategy: tokenizers::PaddingStrategy::Fixed(max_tokens),
        //     direction: tokenizers::PaddingDirection::Right,
        //     pad_to_multiple_of: None,
        //     pad_id: 0,
        //     pad_type_id: 0,
        //     pad_token: "[PAD]".to_string(),
        // };
        // tokenizer.with_padding(Some(padding));
        Ok(Self
        {
            dimension,
            max_tokens,
            overlap_tokens: (max_tokens / 10).min(256), // Максимум 256 токенов перекрытия,
            model_name: model_name,
            config,
            device,
            tokenizer
        })
    }
    async fn load_model(&self) -> Result<BertModel>
    {
        let start = std::time::Instant::now();
        let config = self.config().clone();
        let model_name = self.model_name.as_ref().to_owned();
        let device = self.device().clone();
        let result = tokio::task::spawn_blocking(move ||
        {
            let vb = VarBuilder::from_pth("./model/pytorch_model.bin", DTYPE, &device)?;
            // Use tanh based approximation for Gelu instead of erf implementation.
            // if self.approximate_gelu {
            //     config.hidden_act = HiddenAct::GeluApproximate;
            // }
            let model = BertModel::load(vb, &config)?;
            info!("model {} loaded in {:?}", model_name, start.elapsed());
            Ok(model)
        }).await?;
        result
    }
    pub fn tokenizer(&self) -> &Tokenizer
    {
        &self.tokenizer
    }
    pub fn tokenizer_mut(&mut self) -> &mut Tokenizer
    {
        &mut self.tokenizer
    }
    pub fn device(&self) -> &Device
    {
        &self.device
    }
    pub fn config(&self) -> &Config
    {
        &self.config
    }
    pub async fn model(&self) -> Result<&BertModel>
    {
        if let Some(model) = MODEL.get()
        {
            Ok(model)
        }
        else 
        {
            let model = self.load_model().await.inspect_err(|e| error!("{}", e))?;
            let _ = MODEL.set(model);
            Ok(MODEL.get().unwrap())
        }
    }
}

#[cfg(test)]
mod tests
{
    use candle_transformers::models::bert::DTYPE;
    use candle_core::Tensor;
    use tokenizers::PaddingParams;
    use tracing::info;

    use crate::context_model::ContextModel;
    use crate::error::{Error, Result};
    use crate::logger;
    #[tokio::test]
    async fn test_model()
    {
        logger::init();
        let start = std::time::Instant::now();
        let mut context = super::ContextModel::new(crate::context_model::ModelName::M3).await.unwrap();
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
        if let Some(pp) = context.tokenizer_mut().get_padding_mut() 
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
            context.tokenizer_mut().with_padding(Some(pp));
        }
        let tokens = context.tokenizer()
            .encode_batch(sentences.to_vec(), true).unwrap();

        let token_ids = tokens
            .iter()
            .map(|tokens| {
                let tokens = tokens.get_ids().to_vec();
                Ok(Tensor::new(tokens.as_slice(), context.device()).unwrap())
            })
            .collect::<Result<Vec<_>>>().unwrap();

        let attention_mask = tokens
            .iter()
            .map(|tokens| {
                let tokens = tokens.get_attention_mask().to_vec();
                Ok(Tensor::new(tokens.as_slice(), context.device()).unwrap())
            })
            .collect::<Result<Vec<_>>>().unwrap();

        let token_ids = Tensor::stack(&token_ids, 0).unwrap();
        let attention_mask = Tensor::stack(&attention_mask, 0).unwrap();
        let token_type_ids = token_ids.zeros_like().unwrap();
        info!("running inference on batch {:?}", token_ids.shape());
        let embeddings = context.model().await.unwrap().forward(&token_ids, &token_type_ids, Some(&attention_mask)).unwrap();
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

    pub fn normalize_l2(v: &Tensor) -> Result<Tensor> 
    {
        Ok(v.broadcast_div(&v.sqr().unwrap().sum_keepdim(1).unwrap().sqrt().unwrap()).unwrap())
    }

    fn mean_pooling(embeddings: &Tensor, attention_mask: &Tensor) -> Result<Tensor> 
    {
        let attention_mask_for_pooling = attention_mask.to_dtype(DTYPE)?.unsqueeze(2)?;
        let sum_mask = attention_mask_for_pooling.sum(1)?;
        let embeddings = (embeddings.broadcast_mul(&attention_mask_for_pooling)?).sum(1)?;
        let result = embeddings.broadcast_div(&sum_mask)?;
        let result = result.broadcast_div(&result.sqr()?.sum_keepdim(1)?.sqrt()?)?;
        Ok(result)
    }
}