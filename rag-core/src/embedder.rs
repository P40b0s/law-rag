
pub trait Embedder
{
    /// Генерирует эмбеддинги для батча текстов за один forward pass.
    ///
    /// # Аргументы
    /// * `texts` - Слайс строк для генерации эмбеддингов
    ///
    /// # Возвращает
    /// * `Ok(Vec<Vec<f32>>)` - Вектор эмбеддингов для каждого текста
    fn generate_embeddings_batch(&self, texts: &[&str]) -> impl std::future::Future<Output = anyhow::Result<Vec<Vec<f32>>>> + Send;

    /// Генерирует вектор эмбеддингов для одного текста.
    ///
    /// # Аргументы
    /// * `text` - Текст для ембеддинга
    ///
    /// # Возвращает
    /// * `Ok(Vec<f32>)` - Вектор эмбеддингов (размерность = self.dimension)
    /// * `Err` - Если произошла ошибка или количество текстов != 1
    ///
    /// # Примечание
    /// Для батчевой обработки используйте `generate_embeddings_batch`.
    fn generate_embeddings(&self, text: &str) -> impl std::future::Future<Output = anyhow::Result<Vec<f32>>> + Send;
    // Обычно для BERT hidden_size = 1024, но может отличаться в зависимости от модели
    fn dimension(&self) -> usize;
}