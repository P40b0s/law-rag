
pub trait Encoder
{
    fn decode(&self, ids: &[u32], skip_special_tokens: bool) -> anyhow::Result<String>;
    fn encode(&self, text: &str, add_special_tokens: bool) -> anyhow::Result<tokenizers::Encoding>;
    fn max_tokens(&self) -> usize;
    fn overlap_tokens(&self) -> usize;
    fn tokenizer(&self) -> &tokenizers::Tokenizer;
}