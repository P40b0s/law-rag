use snafu::{AsErrorSource, Snafu, Whatever};
use std::error::Error as StdError;
pub type Result<T, E = Error> = std::result::Result<T, E>;
//impl AsErrorSource for dyn Error + 'static {}
//impl AsErrorSource for dyn Error + Send + 'static{}
//impl AsErrorSource for dyn Error + Sync + 'static{}
// impl AsErrorSource for dyn StdError + Send + Sync + 'static
// {
//     fn as_error_source(&self) -> &(dyn StdError + 'static) 
//     {
        
//     }
// }
// impl StdError for Error 
// {
//     fn source(&self) -> Option<&(dyn StdError + 'static)> 
//     {
//         Some(self.as_error_source())
//     }
// }

#[derive(Debug, thiserror::Error)]
pub enum Error 
{
    #[error(transparent)]
    Tokenizer
    {
        #[from]
        source: tokenizers::Error
    },
    #[error(transparent)]
    SystemaApiError
    {
        #[from]
        source: systema_client::Error
    },
    #[error("Error {0} while parse document {1}", context, uri)]
    ParserError
    {
        context: String,
        uri: String
    },
    #[error(transparent)]
    QdrantError
    {
        #[from]
        source: qdrant_client::QdrantError
    },
    #[error("Failed to load model {}: {}", model, source)]
    ModelLoadError 
    { 
        model: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("Failed during tokenization of text: {}", text)]
    TokenizationError 
    {
        text: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("Tensor operation failed: {}", source)]
    TensorError 
    {
        #[from]
        source: candle_core::Error,
    },
    
    #[error("Model inference failed: {}", source)]
    InferenceError 
    {
        source: candle_core::Error,
    },
    
    #[error("No hidden states in model output")]
    NoHiddenStates,
    
    #[error("Model not loaded: {}", model_name)]
    ModelNotLoaded 
    {
        model_name: String,
    },
}