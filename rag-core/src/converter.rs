pub trait Converter<T>
{
    fn convert(&self, html: String) -> T;
}