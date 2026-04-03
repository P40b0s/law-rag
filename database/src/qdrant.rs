use std::str::FromStr;
use std::sync::Arc;

use anyhow::anyhow;
use embedding::{BgeReranker, Model};
use rag_core::{Chunk, Embedder, EmbeddingConfiguration, QdrantConfiguration, RerankResult, ServiceStatus};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use uuid::Uuid;
use qdrant_client::config::QdrantConfig;
use qdrant_client::qdrant::{
    Condition, CreateCollectionBuilder, CreateFieldIndexCollectionBuilder, FieldCondition, FieldType, Filter, PointId, PointStruct, ScrollPointsBuilder, ScoredPoint, SearchPoints, SearchPointsBuilder, UpdateStatus, UpsertPointsBuilder, Value, WithPayloadSelector
};
use qdrant_client::Qdrant;
use crate::error::{Result, Error};

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
    pub text: String,
    pub document_uri: String,
    pub publication_url: String,
    pub document_hash: String,
    pub document_title: String,
    pub document_number: String,
    /// Дата подписания в формате YYYYMMDD (u64) — для range-фильтров.
    pub document_sign_date: u64,
    pub path: String,
    /// Хэши связанных документов (ссылки из текста чанка).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links_hashes: Vec<String>,
}

impl QdrantPayload
{
    /// Дата подписания в читаемом виде "ДД.ММ.ГГГГ" для передачи в LLM-контекст.
    /// Внутри хранится как u64 YYYYMMDD для range-фильтров Qdrant.
    pub fn sign_date_as_str(&self) -> String
    {
        let d = self.document_sign_date;
        let year  = d / 10_000;
        let month = (d % 10_000) / 100;
        let day   = d % 100;
        format!("{:02}.{:02}.{}", day, month, year)
    }
}

#[derive(Debug, Clone)]
pub struct QdrantInternalConfig
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
impl FromStr for Distance
{
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> 
    {
        match s
        {
            "cosine" => Ok(Distance::Cosine),
            "euclidean" => Ok(Distance::Euclidean),
            "dot" => Ok(Distance::Dot),
            _ => Err(anyhow!("{} is not valid distance, use `cosine`, `euclidean` or `dot`", s))
        }
    }
}


pub struct QdrantManager
{
    client: Qdrant,
    qdrant_config: Arc<QdrantConfiguration>,
    embedding_config: Arc<EmbeddingConfiguration>,
}

impl QdrantManager
{
    pub  fn new(qdrant_config: Arc<QdrantConfiguration>, embedding_config: Arc<EmbeddingConfiguration>) -> Result<Self> 
    {
        let client = QdrantConfig::from_url(&qdrant_config.qdrant_url).connect_timeout(std::time::Duration::from_secs(5)).build()?;
        Ok(Self 
        {
            client,
            qdrant_config,
            embedding_config,
        })
    }

    pub fn configuration(&self) -> &QdrantConfiguration
    {
        &self.qdrant_config
    }

    pub async fn get_collections_names(&self) -> Result<Vec<String>>
    {
        let collections_list = self.client.list_collections().await?;
        Ok(collections_list.collections
            .into_iter()
            .map(|c| c.name)
            .collect()
        )
    }
    /// Создание коллекции (если не существует)
    pub async fn ensure_collection(&self, collection_name: &str, dimension: usize) -> Result<()> 
    {
        let collections_list = self.client.list_collections().await?;
        let distance: Distance = self.qdrant_config.distance.parse()?;
        if !collections_list.collections.iter()
            .any(|c| c.name == collection_name) 
        {

            let vec_params =   qdrant_client::qdrant::VectorParamsBuilder::new(dimension as u64, distance.into()).build();
            let vectors_config = qdrant_client::qdrant::VectorsConfig
            {
                config: Some(qdrant_client::qdrant::vectors_config::Config::Params(vec_params))
                      
            };
            let collection = CreateCollectionBuilder::new(collection_name)
            .vectors_config(vectors_config).build();
            let _ = self.client.create_collection(collection).await?;
            
            
            info!("Created collection: {}", collection_name);

            // Создаём payload-индексы для полей, используемых в фильтрах.
            // Keyword-индекс даёт O(log n) вместо full-scan при фильтрации.
            let keyword_fields = ["document_hash", "document_number"];
            for field in keyword_fields
            {
                let _ = self.client.create_field_index(
                    CreateFieldIndexCollectionBuilder::new(
                        collection_name,
                        field,
                        FieldType::Keyword,
                    ).build()
                ).await
                .inspect_err(|e| warn!("Failed to create keyword index for '{}': {}", field, e));
            }

            // Integer-индекс для range-фильтра по дате.
            let _ = self.client.create_field_index(
                CreateFieldIndexCollectionBuilder::new(
                    collection_name,
                    "document_sign_date",
                    FieldType::Integer,
                ).build()
            ).await
            .inspect_err(|e| warn!("Failed to create integer index for 'document_sign_date': {}", e));
        }
        else
        {
            warn!("Collection already exists: {}", collection_name);
        }

        Ok(())
    }
    

    /// Добавление чанков с эмбеддингами в Qdrant (батчевая обработка)
    pub async fn add_chunks_to_qdrant<'model>(
        &self,
        chunks: Vec<Chunk>,
        embedder: &'model impl Embedder,
        collection_name: &str,
        sender: tokio::sync::mpsc::UnboundedSender<ServiceStatus>
    ) -> Result<Vec<String>> {
        use tokio::time::{timeout, Duration};

        let mut all_ids = Vec::new();
        let chunks_count = chunks.len();
        let doc_hash = chunks.first().map(|c| c.hash.clone()).unwrap_or(String::new());
        let dimension = embedder.dimension();
        let batch_size = self.embedding_config.embedding_batch_size;
        let process = ServiceStatus::process_qdrant(
            &doc_hash,
            0,
            chunks_count
        );
        let _ = sender.send(process);
        info!("Starting to add {} chunks to Qdrant for document {} (batch_size={})",
            chunks_count, doc_hash, batch_size);
      
        
        let start_time = std::time::Instant::now();

        // Обрабатываем мини-батчами
        for (batch_idx, batch) in chunks.chunks(batch_size).enumerate()
        {
            let batch_start = batch_idx * batch_size;
            let batch_end = (batch_start + batch.len()).min(chunks_count);

            let elapsed = start_time.elapsed();
            let processed = batch_start;
            let chunks_per_sec = if elapsed.as_secs() > 0 
            {
                processed as f64 / elapsed.as_secs_f64()
            } 
            else 
            {
                0.0
            };
            let eta_secs = if chunks_per_sec > 0.0 
            {
                ((chunks_count - processed) as f64 / chunks_per_sec) as u64
            }
            else 
            {
                0
            };

            info!("Processing batch {}/{} (chunks {}-{}/{}, {:.2} chunks/sec, ETA: {}s)",
                batch_idx + 1,
                (chunks_count + batch_size - 1) / batch_size,
                batch_start + 1, batch_end, chunks_count,
                chunks_per_sec, eta_secs
            );
           

            // Собираем тексты для батчевого эмбеддинга
            let texts: Vec<&str> = batch.iter().map(|c| c.content.as_str()).collect();

            // Генерируем эмбеддинги для всего батча за один forward pass
            let embedding_timeout = Duration::from_secs(300);
            let embeddings_result = timeout(
                embedding_timeout,
                embedder.generate_embeddings_batch(&texts)
            ).await;

            let embeddings = match embeddings_result 
            {
                Ok(Ok(embs)) => 
                {
                    info!("Generated {} embeddings for batch {}", embs.len(), batch_idx + 1);
                    embs
                },
                Ok(Err(e)) => 
                {
                    error!("Error generating embeddings for batch {}: {}", batch_idx + 1, e);
                    let status = ServiceStatus::error(doc_hash.clone(), e);
                    let _ = sender.send(status);
                    return Err(Error::AnyhowError(anyhow::anyhow!(
                        "Error generating embeddings for batch {}", batch_idx + 1
                    )));
                },
                Err(_) => 
                {
                    error!("Timeout generating embeddings for batch {} after {:?}",
                        batch_idx + 1, embedding_timeout);
                    let status = ServiceStatus::error(
                        doc_hash.clone(),
                        anyhow::anyhow!("Timeout generating embeddings for batch {}", batch_idx + 1)
                    );
                    let _ = sender.send(status);
                    return Err(Error::AnyhowError(anyhow::anyhow!(
                        "Timeout generating embeddings for batch {}", batch_idx + 1
                    )));
                }
            };

            if embeddings.len() != batch.len() 
            {
                let error = format!("Embeddings count mismatch: expected {}, got {}", batch.len(), embeddings.len());
                error!("{}", &error);
                let status = ServiceStatus::error(doc_hash.clone(), error);
                let _ = sender.send(status);
                return Err(Error::AnyhowError(anyhow::anyhow!(
                    "Embeddings count mismatch: expected {}, got {}", batch.len(), embeddings.len()
                )));
                 
            }

            // Создаем точки для Qdrant из всего батча
            let mut points = Vec::with_capacity(batch.len());
            for (i, (chunk, embedding)) in batch.iter().zip(embeddings.into_iter()).enumerate()
            {
                let global_idx = batch_start + i;

                // Проверяем размерность
                if embedding.len() != dimension 
                {
                    error!("Vector size mismatch for chunk {}/{}: expected {}, got {}",
                        global_idx + 1, chunks_count, dimension, embedding.len());
                    return Err(Error::VectorSizeError(dimension, embedding.len()));
                }

                let point = self.create_point(chunk.clone(), embedding)?;
                points.push(point);

                // Отправляем прогресс для каждого чанка
                let process = ServiceStatus::process_qdrant(
                    &chunk.hash,
                    global_idx,
                    chunks_count
                );
                let _ = sender.send(process);
            }

            // Upsert всего батча в Qdrant за одну операцию
            let point_structs: Vec<PointStruct> = points.iter()
                .map(|p| p.into())
                .collect();
            let builder = UpsertPointsBuilder::new(collection_name, point_structs);

            let upsert_timeout = Duration::from_secs(60);
            let upsert_result = timeout(
                upsert_timeout,
                self.client.upsert_points(builder.build())
            ).await;

            match upsert_result 
            {
                Ok(Ok(_)) => 
                {
                    info!("Upserted batch {} ({} points) to Qdrant", batch_idx + 1, points.len());
                },
                Ok(Err(e)) => 
                {
                    let error = format!("Error upserting batch {}: {}", batch_idx + 1, e);
                    error!("{}", &error);
                    let status = ServiceStatus::error(doc_hash.clone(), error);
                    let _ = sender.send(status);
                    return Err(e.into());
                },
                Err(_) => 
                {
                    let error = format!("Timeout upserting batch {} after {:?}", batch_idx + 1, upsert_timeout);
                    error!("{}", &error);
                    let status = ServiceStatus::error(doc_hash.clone(), error);
                    let _ = sender.send(status);
                    return Err(Error::AnyhowError(anyhow::anyhow!(
                        "Timeout upserting batch {}", batch_idx + 1
                    )));
                }
            }

            all_ids.extend(points.iter().map(|p| p.id.clone()));
        }

        let total_time = start_time.elapsed();
        info!("Successfully added {} chunks to Qdrant in {:?} ({:.2} chunks/sec)",
            all_ids.len(),
            total_time,
            all_ids.len() as f64 / total_time.as_secs_f64()
        );
        let _ = sender.send(ServiceStatus::complete(&doc_hash));
        Ok(all_ids)
    }
    
    fn create_point(
        &self,
        chunk: Chunk,
        vector: Vec<f32>,
    ) -> Result<QdrantPoint> 
    {
        let id = Uuid::now_v7().to_string();
        
        // JoinDate → "YYYYMMDD", парсим в u64 для range-фильтров в Qdrant.
        let sign_date_u64: u64 = chunk.sign_date
            .format(utilites::DateFormat::JoinDate)
            .parse()
            .unwrap_or(0);
        Ok(QdrantPoint
        {
            id,
            vector,
            payload: QdrantPayload
            {
                text: chunk.content,
                document_uri: chunk.document_url,
                publication_url: chunk.publication_url,
                document_hash: chunk.hash,
                document_title: chunk.title,
                document_number: chunk.number,
                document_sign_date: sign_date_u64,
                path: chunk.path,
                links_hashes: chunk.links_hashes.unwrap_or_default(),
            },
        })
    }
    
    /// Поиск по семантическому сходству
    // pub async fn semantic_search<'model>(
    //     &self,
    //     query: &str,
    //     limit: usize,
    //     rerank_limit: usize,
    //     retriver_model: &'model embedding::RetriverModel,
    //     reranker_model: &'model embedding::BgeReranker,
    //     filter: Option<SearchFilter>,
    // ) -> Result<Vec<RerankResult<SearchResult>>> {
    //     // Получаем эмбеддинг для запроса
    //     let query_vector = retriver_model.generate_embeddings(&[query]).await?;
    //     // Строим запрос к Qdrant
    //     let mut search_request = SearchPoints 
    //     {
    //         collection_name: self.config.collection_name.clone(),
    //         vector: query_vector,
    //         limit: limit as u64,
    //         with_payload: Some(WithPayloadSelector 
    //         {
    //             selector_options: Some(
    //                 qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true)
    //             ),
    //         }),
    //         ..Default::default()
    //     };
        
    //     // Добавляем фильтры, если есть
    //     if let Some(filter) = filter 
    //     {
    //         search_request.filter = Some(filter.into());
    //     }
    //      //let builder = SearchPointsBuilder::new(&self.config.collection_name, search_request, limit);
    //     // Вставляем в Qdrant
    //     //let _ = self.client.upsert_points(builder.build()).await?;
    //     // Выполняем поиск
    //     let search_results = self.client
    //         .search_points(search_request)
    //         .await?;
        
    //     // Конвертируем результаты
    //     let results: Vec<SearchResult> = search_results.result
    //         .into_iter()
    //         .map(|sp| SearchResult::from_scored_point(sp))
    //         .collect();
    //     let reranking = reranker_model.rerank(query, results, rerank_limit).await
    //         .inspect_err(|e| tracing::error!("{}", e))?;
    //     //reranking.into_iter().map(|r| r)
    //     Ok(reranking)
    // }

     /// Поиск по семантическому сходству (если коллекций несколько выносим создание эмбеддингов за цикл)
    /// Векторный поиск без реранкинга — для использования в мульти-коллекционном поиске.
    /// Реранкинг должен выполняться один раз после объединения результатов всех коллекций.
    /// Минимальный cosine score для попадания в пул реранкинга.
    /// Отсекает заведомо нерелевантные результаты до реранкера.

    pub async fn vector_search_raw(
        &self,
        embeddings: Vec<f32>,
        collection_name: &str,
        limit: usize,
        filter: Option<SearchFilter>,
    ) -> Result<Vec<SearchResult>>
    {
        let mut search_request = SearchPoints
        {
            collection_name: collection_name.to_owned(),
            vector: embeddings,
            limit: limit as u64,
            score_threshold: Some(self.qdrant_config.min_search_score),
            with_payload: Some(WithPayloadSelector
            {
                selector_options: Some(
                    qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true)
                ),
            }),
            ..Default::default()
        };
        if let Some(f) = filter
        {
            search_request.filter = Some(f.into());
        }
        let results = self.client.search_points(search_request).await?;
        Ok(results.result.into_iter().map(SearchResult::from_scored_point).collect())
    }

    // pub async fn semantic_search_with_embeddings<'model>(
    //     &self,
    //     query: &str,
    //     limit: usize,
    //     rerank_limit: usize,
    //     embeddings: Vec<f32>,
    //     reranker_model: &'model embedding::BgeReranker,
    //     filter: Option<SearchFilter>,
    // ) -> Result<Vec<RerankResult<SearchResult>>> 
    // {
    //     // Получаем эмбеддинг для запроса
    //     //let query_vector = retriver_model.generate_embeddings(&[query]).await?;
    //     // Строим запрос к Qdrant
    //     let mut search_request = SearchPoints 
    //     {
    //         collection_name: self.config.collection_name.clone(),
    //         vector: embeddings,
    //         limit: limit as u64,
    //         with_payload: Some(WithPayloadSelector 
    //         {
    //             selector_options: Some(
    //                 qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true)
    //             ),
    //         }),
    //         ..Default::default()
    //     };
        
    //     // Добавляем фильтры, если есть
    //     if let Some(filter) = filter 
    //     {
    //         search_request.filter = Some(filter.into());
    //     }
    //      //let builder = SearchPointsBuilder::new(&self.config.collection_name, search_request, limit);
    //     // Вставляем в Qdrant
    //     //let _ = self.client.upsert_points(builder.build()).await?;
    //     // Выполняем поиск
    //     let search_results = self.client
    //         .search_points(search_request)
    //         .await?;
        
    //     // Конвертируем результаты
    //     let results: Vec<SearchResult> = search_results.result
    //         .into_iter()
    //         .map(|sp| SearchResult::from_scored_point(sp))
    //         .collect();
    //     let reranking = reranker_model.rerank(query, results, rerank_limit).await
    //         .inspect_err(|e| tracing::error!("{}", e))?;
    //     //reranking.into_iter().map(|r| r)
    //     Ok(reranking)
    // }
    
    /// Поиск с гибридным подходом (семантический + ключевые слова)
    // pub async fn hybrid_search<'model>(
    //     &self,
    //     query: &str,
    //     limit: usize,
    //     rerank_limit: usize,
    //     filter: Option<SearchFilter>,
    //     keyword_weight: f32,
    //     semantic_weight: f32,
    //     retriver_model: &'model embedding::RetriverModel,
    //     reranker_model: &'model embedding::BgeReranker,
    // ) -> Result<Vec<SearchResult>> {
    //     // 1. Семантический поиск
    //     let semantic_results = self.semantic_search(query, limit * 2, rerank_limit * 2, retriver_model, reranker_model, filter.clone()).await?;
    //     let semantic_results = semantic_results.into_iter()
    //     .map(|r| SearchResult
    //     {
    //         id: r.db_object.id,
    //         score: r.score,
    //         payload: r.db_object.payload
    //     }).collect();
    //     // 2. Поиск по ключевым словам (если нужно)
    //     // Можно использовать BM25 или другие методы
    //     let keyword_results = self.keyword_search(query, limit * 2, filter).await?;
        
    //     // 3. Объединяем результаты
    //     let combined = Self::combine_results(
    //         semantic_results,
    //         keyword_results,
    //         semantic_weight,
    //         keyword_weight,
    //         limit,
    //     );
        
    //     Ok(combined)
    // }
    
    /// Поиск по ключевым словам через Qdrant scroll (без вектора).
    /// Использует text-match по полю `text` + дополнительные фильтры.
    async fn keyword_search(
        &self,
        query: &str,
        limit: usize,
        collection_name: &str,
        filter: Option<SearchFilter>,
    ) -> Result<Vec<SearchResult>>
    {
        let keywords = Self::extract_keywords(query);

        // Text-match условия (should = OR между словами)
        let keyword_conditions: Vec<Condition> = keywords
            .into_iter()
            .map(|kw| Condition
            {
                condition_one_of: Some(
                    qdrant_client::qdrant::condition::ConditionOneOf::Field(
                        FieldCondition
                        {
                            key: "text".to_string(),
                            r#match: Some(qdrant_client::qdrant::Match
                            {
                                match_value: Some(
                                    qdrant_client::qdrant::r#match::MatchValue::Text(kw)
                                ),
                            }),
                            ..Default::default()
                        }
                    )
                ),
            })
            .collect();

        if keyword_conditions.is_empty()
        {
            return Ok(Vec::new());
        }

        let mut scroll_filter = Filter
        {
            should: keyword_conditions,
            ..Default::default()
        };

        // Дополнительные must-фильтры (например, по document_hash)
        if let Some(f) = filter
        {
            scroll_filter.must = f.conditions;
        }

        let scroll = ScrollPointsBuilder::new(collection_name)
            .filter(scroll_filter)
            .limit(limit as u32)
            .with_payload(true)
            .build();

        let response = self.client.scroll(scroll).await?;

        Ok(response.result
            .into_iter()
            .map(SearchResult::from_retrieved_point)
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
    pub async fn get_document_chunks<'model>(
        &self,
        hash: &str,
        limit: Option<usize>,
        dimension: usize,
    ) -> Result<Vec<SearchResult>> 
    {
        let filter = SearchFilter::new()
            .add_exact_match("document_hash", hash);
        let collections_list = self.client.list_collections().await?;
        let mut search_results = None;
        for collection in collections_list.collections
        {
            let search_request = SearchPoints 
            {
                collection_name: collection.name,
                vector: vec![0.0; dimension],
                filter: Some(filter.clone().into()),
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
            if results.result.len() > 0
            {
                search_results = Some(results);
                break;
            }
        }
        
        if let Some(results) = search_results
        {
            Ok(results.result
            .into_iter()
            .map(SearchResult::from_scored_point)
            .collect())
        }
        else
        {
            Ok(Vec::with_capacity(0))
        }
    }
    
    /// Удаление документа из индекса
    pub async fn delete_document(&self, hash: &str) -> Result<Option<UpdateStatus>> 
    {
        let filter = SearchFilter::new()
            .add_exact_match("document_hash", hash);
        let collections_list = self.client.list_collections().await?;
        let mut update_status = None;
        for collection in collections_list.collections
        {
             let delete_request = qdrant_client::qdrant::DeletePoints 
            {
                collection_name: collection.name,
                points: Some(qdrant_client::qdrant::PointsSelector 
                {
                    points_selector_one_of: Some(
                        qdrant_client::qdrant::points_selector::PointsSelectorOneOf::Filter(
                            filter.clone().into()
                        )
                    ),
                }),
                ..Default::default()
            };
        
            let response = self.client
                .delete_points(delete_request)
                .await?;
            if let Some(status) = response.result
            {
                let status = status.status();
                update_status = Some(status.clone());
                if let UpdateStatus::Completed = status
                {
                    break;
                }
            }
        }
        Ok(update_status)
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
    fn parse_payload(raw: std::collections::HashMap<String, Value>) -> QdrantPayload
    {
        serde_json::from_value(
            serde_json::to_value(raw).unwrap()
        ).unwrap_or_else(|_| QdrantPayload
        {
            text: String::new(),
            document_uri: String::new(),
            publication_url: String::new(),
            document_title: String::new(),
            document_hash: String::new(),
            path: String::new(),
            document_number: String::new(),
            document_sign_date: 0,
            links_hashes: Vec::new(),
        })
    }

    fn parse_point_id(id: Option<PointId>) -> String
    {
        id.and_then(|a| a.point_id_options.map(|p| match p
        {
            qdrant_client::qdrant::point_id::PointIdOptions::Num(n) => n.to_string(),
            qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uid) => uid,
        }))
        .unwrap_or_default()
    }

    fn from_scored_point(sp: ScoredPoint) -> Self
    {
        Self
        {
            id: Self::parse_point_id(sp.id),
            score: sp.score,
            payload: Self::parse_payload(sp.payload),
        }
    }

    fn from_retrieved_point(rp: qdrant_client::qdrant::RetrievedPoint) -> Self
    {
        Self
        {
            id: Self::parse_point_id(rp.id),
            score: 0.0,
            payload: Self::parse_payload(rp.payload),
        }
    }
}

impl ToString for SearchResult 
{
    fn to_string(&self) -> String 
    {
        format!(
        "[DOC id={}]
        title: {}
        date: {}
        number: {}
        uri: {}
        path: {}
        
        text: {}
        [/DOC]\n\n",
            self.payload.document_hash,
            self.payload.document_title,
            self.payload.sign_date_as_str(),
            self.payload.document_number,
            self.payload.document_uri,
            self.payload.path,
            self.payload.text
        )
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
