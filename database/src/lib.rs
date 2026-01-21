mod error;

use embedding::{BgeReranker, Chunk, RerankResult};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;
use qdrant_client::qdrant::{
    Condition, CreateCollectionBuilder, FieldCondition, Filter, PointId, PointStruct, ScoredPoint, SearchPoints, SearchPointsBuilder, UpsertPointsBuilder, Value, WithPayloadSelector
};
use qdrant_client::Qdrant;
use crate::error::Result;
pub use error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QdrantPoint 
{
    pub id: String, // Используем Uuid в виде строки
    pub vector: Vec<f32>,
    pub payload: QdrantPayload,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd)]
pub struct QdrantPayload 
{
    pub text: String,                // Оригинальный текст чанка
    //pub embedding_text: String,      // Текст, который был передан в модель
    pub document_uri: String,        // URI исходного документа
    pub document_title: String, // Заголовки документа
    pub document_number: String,    // Номер документа
    pub document_sign_date: String, // Дата подписания документа
    pub path: String, // Полный путь
    pub chunk_index: usize,          // Индекс чанка в документе
}

#[derive(Debug, Clone)]
pub struct QdrantConfig 
{
    pub url: String,
    pub collection_name: String,
    pub distance: Distance, // Метрика расстояния
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Distance 
{
    Cosine,
    Euclidean,
    Dot,
}

impl From<Distance> for qdrant_client::qdrant::Distance 
{
    fn from(d: Distance) -> Self 
    {
        match d 
        {
            Distance::Cosine => Self::Cosine,
            Distance::Euclidean => Self::Euclid,
            Distance::Dot => Self::Dot,
        }
    }
}
impl From<Distance> for i32
{
    fn from(value: Distance) -> Self 
    {
        match value 
        {
            Distance::Cosine => 0,
            Distance::Euclidean => 1,
            Distance::Dot => 2
        }
    }
}


pub struct QdrantManager 
{
    client: Qdrant,
    config: QdrantConfig,
    embedding_client: embedding::Embeddings,
    reranking_client: embedding::BgeReranker
}

impl QdrantManager 
{
    pub async fn new(config: QdrantConfig, embedding_client: embedding::Embeddings, reranking_client: BgeReranker) -> Result<Self> 
    {
        let client = Qdrant::from_url(&config.url)
            .build()?;
            
        Ok(Self 
        {
            client,
            config,
            embedding_client,
            reranking_client
        })
    }
    
    /// Создание коллекции (если не существует)
    pub async fn ensure_collection(&self) -> Result<()> 
    {
        let collections_list = self.client.list_collections().await?;
        
        if !collections_list.collections.iter()
            .any(|c| c.name == self.config.collection_name) 
        {

            let vec_params =   qdrant_client::qdrant::VectorParamsBuilder::new(self.embedding_client.dimension() as u64, self.config.distance.clone().into()).build();
            let vectors_config = qdrant_client::qdrant::VectorsConfig
            {
                config: Some(qdrant_client::qdrant::vectors_config::Config::Params(vec_params))
                      
            };
            let collection = CreateCollectionBuilder::new(&self.config.collection_name)
            .vectors_config(vectors_config).build();
            let _ = self.client.create_collection(collection).await?;
            
            
            info!("Created collection: {}", self.config.collection_name);
        } 
        else 
        {
            warn!("Collection already exists: {}", self.config.collection_name);
        }
        
        Ok(())
    }
    
    /// Добавление чанков с эмбеддингами в Qdrant
    pub async fn add_chunks_to_qdrant(
        &self,
        chunks: Vec<Chunk>
    ) -> Result<Vec<String>> {
        let mut all_ids = Vec::new();
        
        // Обрабатываем батчами для эффективности
        for chunk in chunks
        {
            let batch_ids = self.process_batch(chunk).await?;
            all_ids.extend(batch_ids);
        }
        
        info!("Added {} chunks to Qdrant", all_ids.len());
        Ok(all_ids)
    }
    
    async fn process_batch(&self, chunk: Chunk) -> Result<Vec<String>> 
    {
        let mut points = Vec::new();
        let dimension = self.embedding_client.dimension();
        
        // Получаем эмбеддинг для чанка
        let embeddings = self.embedding_client.generate_embeddings(&[chunk.content.as_str()]).await?;
        // Проверяем размерность
        if embeddings.len() != dimension 
        {
            return Err(Error::VectorSizeError(dimension, embeddings.len()));
        }
        // Создаем точку для Qdrant
        let point = self.create_point(chunk, embeddings)?;
        points.push(point);

        // Конвертируем в PointStruct для Qdrant
        let point_structs: Vec<PointStruct> = points.iter()
            .map(|p| p.into())
            .collect();
        let builder = UpsertPointsBuilder::new(&self.config.collection_name, point_structs);
        // Вставляем в Qdrant
        let _ = self.client.upsert_points(builder.build()).await?;
        
        Ok(points.iter().map(|p| p.id.clone()).collect())
    }
    
    fn create_point(
        &self,
        chunk: Chunk,
        vector: Vec<f32>,
    ) -> Result<QdrantPoint> 
    {
        let id = Uuid::new_v4().to_string();
        
        Ok(QdrantPoint 
            {
                id,
                vector,
                payload: QdrantPayload 
                {
                    text: chunk.content,
                    document_uri: chunk.document_url,
                    document_title: chunk.title,
                    document_number: chunk.number,
                    document_sign_date: chunk.sign_date.to_string(),
                    path: chunk.path,
                    chunk_index: chunk.meta.and_then(|m| Some(m.chunk_index)).unwrap_or(0),
                },
        })
    }
    
    /// Поиск по семантическому сходству
    pub async fn semantic_search(
        &self,
        query: &str,
        limit: usize,
        filter: Option<SearchFilter>,
    ) -> Result<Vec<RerankResult<SearchResult>>> {
        // Получаем эмбеддинг для запроса
        let query_vector = self.embedding_client.generate_embeddings(&[query]).await?;
        // Строим запрос к Qdrant
        let mut search_request = SearchPoints 
        {
            collection_name: self.config.collection_name.clone(),
            vector: query_vector,
            limit: limit as u64,
            with_payload: Some(WithPayloadSelector 
            {
                selector_options: Some(
                    qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true)
                ),
            }),
            ..Default::default()
        };
        
        // Добавляем фильтры, если есть
        if let Some(filter) = filter 
        {
            search_request.filter = Some(filter.into());
        }
         //let builder = SearchPointsBuilder::new(&self.config.collection_name, search_request, limit);
        // Вставляем в Qdrant
        //let _ = self.client.upsert_points(builder.build()).await?;
        // Выполняем поиск
        let search_results = self.client
            .search_points(search_request)
            .await?;
        
        // Конвертируем результаты
        let results: Vec<SearchResult> = search_results.result
            .into_iter()
            .map(|sp| SearchResult::from_scored_point(sp))
            .collect();
        let reranking = self.reranking_client.rerank(query, results, 5).await.inspect_err(|e| tracing::error!("{}", e))?;
        //reranking.into_iter().map(|r| r)
        Ok(reranking)
    }
    
    /// Поиск с гибридным подходом (семантический + ключевые слова)
    pub async fn hybrid_search(
        &self,
        query: &str,
        limit: usize,
        filter: Option<SearchFilter>,
        keyword_weight: f32,
        semantic_weight: f32,
    ) -> Result<Vec<SearchResult>> {
        // 1. Семантический поиск
        let semantic_results = self.semantic_search(query, limit * 2, filter.clone()).await?;
        let semantic_results = semantic_results.into_iter()
        .map(|r| SearchResult
        {
            id: r.payload.id,
            score: r.score,
            payload: r.payload.payload
        }).collect();
        // 2. Поиск по ключевым словам (если нужно)
        // Можно использовать BM25 или другие методы
        let keyword_results = self.keyword_search(query, limit * 2, filter).await?;
        
        // 3. Объединяем результаты
        let combined = Self::combine_results(
            semantic_results,
            keyword_results,
            semantic_weight,
            keyword_weight,
            limit,
        );
        
        Ok(combined)
    }
    
    /// Поиск по ключевым словам (используя фильтры Qdrant)
    async fn keyword_search(
        &self,
        query: &str,
        limit: usize,
        filter: Option<SearchFilter>,
    ) -> Result<Vec<SearchResult>> {
        // Извлекаем ключевые слова из запроса
        let keywords = Self::extract_keywords(query);
        
        // Создаем условия для поиска
        let mut conditions = Vec::new();
        for keyword in keywords {
            conditions.push(Condition {
                condition_one_of: Some(
                    qdrant_client::qdrant::condition::ConditionOneOf::Field(
                        FieldCondition {
                            key: "text".to_string(),
                            r#match: Some(qdrant_client::qdrant::Match {
                                match_value: Some(
                                    qdrant_client::qdrant::r#match::MatchValue::Text(keyword)
                                ),
                            }),
                            ..Default::default()
                        }
                    )
                ),
            });
        }
        
        // Если есть дополнительные фильтры
        let mut all_conditions = conditions;
        if let Some(filter) = filter 
        {
            all_conditions.extend(filter.conditions);
        }
        
        // Выполняем поиск
        let search_request = SearchPoints 
        {
            collection_name: self.config.collection_name.clone(),
            vector: vec![0.0; self.embedding_client.dimension()], // Пустой вектор для keyword search
            filter: Some(Filter 
            {
                should: all_conditions,
                ..Default::default()
            }),
            limit: limit as u64,
            with_payload: Some(WithPayloadSelector 
            {
                selector_options: Some(
                    qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true)
                ),
            }),
            ..Default::default()
        };
        
        let results = self.client
            .search_points(search_request)
            .await?;
        
        Ok(results.result
            .into_iter()
            .map(SearchResult::from_scored_point)
            .collect())
    }
    
    fn extract_keywords(query: &str) -> Vec<String> 
    {
        // Простая реализация - убираем стоп-слова
        let stop_words = vec!["и", "в", "на", "с", "по", "для", "что", "это"];
        query.split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|w| !stop_words.contains(&w.to_lowercase().as_str()))
            .filter(|w| w.len() > 2)
            .map(String::from)
            .collect()
    }
    
    fn combine_results(
        semantic: Vec<SearchResult>,
        keyword: Vec<SearchResult>,
        semantic_weight: f32,
        keyword_weight: f32,
        limit: usize,
    ) -> Vec<SearchResult> 
    {
        use std::collections::HashMap;
        
        let mut scores = HashMap::new();
        let mut results = HashMap::new();
        
        // Собираем семантические результаты
        for result in semantic 
        {
            let score = result.score * semantic_weight;
            scores.insert(result.id.clone(), score);
            results.insert(result.id.clone(), result);
        }
        
        // Добавляем keyword результаты
        for result in keyword 
        {
            let entry = scores.entry(result.id.clone()).or_insert(0.0);
            *entry += result.score * keyword_weight;
            results.entry(result.id.clone())
                .or_insert_with(|| result.clone());
        }
        
        // Сортируем по итоговому скорингу
        let mut combined: Vec<SearchResult> = results.into_values().collect();
        combined.sort_by(|a, b| 
        {
            let score_a = scores.get(&a.id).unwrap_or(&0.0);
            let score_b = scores.get(&b.id).unwrap_or(&0.0);
            score_b.partial_cmp(score_a).unwrap()
        });
        
        combined.into_iter().take(limit).collect()
    }
    
    /// Получение всех чанков документа
    pub async fn get_document_chunks(
        &self,
        document_uri: &str,
        limit: Option<usize>,
    ) -> Result<Vec<SearchResult>> 
    {
        let filter = SearchFilter::new()
            .add_exact_match("document_uri", document_uri);
        
        // Используем пустой вектор для получения всех точек
        let search_request = SearchPoints {
            collection_name: self.config.collection_name.clone(),
            vector: vec![0.0; self.embedding_client.dimension()],
            filter: Some(filter.into()),
            limit: limit.unwrap_or(1000) as u64,
            with_payload: Some(WithPayloadSelector 
            {
                selector_options: Some(
                    qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true)
                ),
            }),
            ..Default::default()
        };
        
        let results = self.client
            .search_points(search_request)
            .await?;
        
        Ok(results.result
            .into_iter()
            .map(SearchResult::from_scored_point)
            .collect())
    }
    
    /// Удаление документа из индекса
    pub async fn delete_document(&self, document_uri: &str) -> Result<Option<u64>> 
    {
        let filter = SearchFilter::new()
            .add_exact_match("document_uri", document_uri);
        
        let delete_request = qdrant_client::qdrant::DeletePoints 
        {
            collection_name: self.config.collection_name.clone(),
            points: Some(qdrant_client::qdrant::PointsSelector 
            {
                points_selector_one_of: Some(
                    qdrant_client::qdrant::points_selector::PointsSelectorOneOf::Filter(
                        filter.into()
                    )
                ),
            }),
            ..Default::default()
        };
        
        let response = self.client
            .delete_points(delete_request)
            .await?;
        Ok(response.result.and_then(|r| r.operation_id))
    }
}

// Вспомогательные структуры
#[derive(Debug, Clone, Default)]
pub struct SearchFilter {
    conditions: Vec<Condition>,
}

impl SearchFilter {
    pub fn new() -> Self {
        Self { conditions: Vec::new() }
    }
    
    pub fn add_exact_match(mut self, key: &str, value: &str) -> Self 
    {
        self.conditions.push(Condition 
            {
            condition_one_of: Some(
                qdrant_client::qdrant::condition::ConditionOneOf::Field(
                    FieldCondition 
                    {
                        key: key.to_string(),
                        r#match: Some(qdrant_client::qdrant::Match 
                        {
                            match_value: Some(
                                qdrant_client::qdrant::r#match::MatchValue::Keyword(value.to_string())
                            ),
                        }),
                        ..Default::default()
                    }
                )
            ),
        });
        self
    }
    
    pub fn add_range(mut self, key: &str, gt: Option<f64>, lt: Option<f64>) -> Self 
    {
        self.conditions.push(Condition 
        {
            condition_one_of: Some(
                qdrant_client::qdrant::condition::ConditionOneOf::Field(
                    FieldCondition 
                    {
                        key: key.to_string(),
                        range: Some(qdrant_client::qdrant::Range 
                        {
                            gt,
                            lt,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }
                )
            ),
        });
        self
    }
}

impl From<SearchFilter> for Filter 
{
    fn from(f: SearchFilter) -> Self 
    {
        Filter 
        {
            must: f.conditions,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult 
{
    pub id: String,
    pub score: f32,
    pub payload: QdrantPayload,
}

impl AsRef<str> for SearchResult
{
    fn as_ref(&self) -> &str 
    {
        &self.payload.text
    }
}



impl SearchResult 
{
    fn from_scored_point(sp: ScoredPoint) -> Self 
    {
        let payload_value = sp.payload;
        let payload: QdrantPayload = serde_json::from_value(
            serde_json::to_value(payload_value).unwrap()
        ).unwrap_or_else(|_| QdrantPayload 
        {
            text: String::new(),
            document_uri: String::new(),
            document_title: String::new(),
            path: String::new(),
            chunk_index: 0,
            document_number: String::new(),
            document_sign_date: String::new()
            
        });
        let point_id = sp.id.and_then(|a| a.point_id_options.and_then(|p|
        {
            let res = match p
            {
                qdrant_client::qdrant::point_id::PointIdOptions::Num(n) => n.to_string(),
                qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uid) => uid
            };
            Some(res)
        }));
        Self 
        {
            id: point_id.unwrap_or(String::new()),
            score: sp.score,
            payload,
        }
    }
}

impl From<&QdrantPoint> for PointStruct 
{
    fn from(point: &QdrantPoint) -> Self 
    {
        let payload: std::collections::HashMap<String, Value> = 
            serde_json::from_value(
                serde_json::to_value(&point.payload).unwrap()
            ).unwrap();
        
        PointStruct 
        {
            id: Some(point.id.clone().into()),
            vectors: Some(point.vector.clone().into()),
            payload,
        }
    }
}
