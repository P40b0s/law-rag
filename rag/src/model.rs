use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LongContextModel {
    // Модели с длинным контекстом
    BgeReranker,           // 8192 токенов
    Bge,              // 8192 токенов
}

impl LongContextModel 
{
    pub fn dimension(&self) -> usize 
    {
        match self {
            Self::Bge => 1024,      // BGE-M3: 1024 размерность
            Self::BgeReranker => 1024,
        }
    }
    
    pub fn max_tokens(&self) -> usize 
    {
        match self {
            Self::Bge => 8192,      // BGE-M3: до 8192 токенов
            Self::BgeReranker => 8192,
        }
    }
    
    pub fn model_name(&self) -> &'static str {
        match self {
            Self::Bge => "BAAI/bge-m3",
            Self::BgeReranker => "BAAI/bge-reranker-v2-m3",
        }
    }
    
    pub fn requires_instruction(&self) -> bool {
        match self {
            Self::Bge => true,  // BGE-M3 требует инструкций
            Self::BgeReranker => true,
            _ => false,
        }
    }
}