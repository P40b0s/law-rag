
#[derive(Debug, Clone)]
pub struct RerankResult<P: ToString + AsRef<str>>
{
    pub score: f32,
    pub db_object: P,
}
impl<P: ToString + AsRef<str>> ToString for RerankResult<P>
{
    fn to_string(&self) -> String 
    {
        self.db_object.to_string()
    }
}
impl<P: ToString + AsRef<str>> AsRef<str> for RerankResult<P>
{
    fn as_ref(&self) -> &str 
    {
        self.db_object.as_ref()
    }
}