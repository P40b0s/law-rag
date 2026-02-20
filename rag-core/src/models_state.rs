use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct GeneratorState
{
    model_name: String,
    model_size: usize,
    system_prompt: String,
    uri: Option<String>,
    temperature: f64
}
#[derive(Debug, Serialize)]
pub struct RetriverState
{
    retriver_model_name: String,
    reranker_model_name: String,
}


#[derive(Debug, Serialize)]
pub struct ModelsState
{
    #[serde(skip_serializing_if="Option::is_none")]
    retriver: Option<RetriverState>,
    #[serde(skip_serializing_if="Option::is_none")]
    generator: Option<GeneratorState>
}

impl ModelsState
{
    pub fn new() -> Self
    {
        Self { retriver: None, generator: None }
    }
    pub fn with_retriver(mut self, retriver_name: String, reranker_name: String) -> Self
    {
        self.retriver = Some(RetriverState
        {
            reranker_model_name: reranker_name,
            retriver_model_name: retriver_name,
        });
        self
    }
    pub fn with_generator<S: AsRef<str>>(mut self, model_name: S, model_size: usize, system_prompt: S, temperature: f64, uri: Option<S>,) -> Self
    {
        self.generator = Some(GeneratorState
        {
            model_name: model_name.as_ref().to_owned(),
            model_size: model_size,
            system_prompt: system_prompt.as_ref().to_owned(),
            uri: uri.map(|u| u.as_ref().to_owned()),
            temperature
        });
        self
    }
}