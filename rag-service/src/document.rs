use database::{CollectionDbo, DocumentCardDbo, DocumentDbo};
use rag_core::{Chunk, DocumentStatus};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use utilites::Date;
use anyhow::Result;
use uuid::Uuid;
use crate::{logger};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Document
{
    pub document_uri: String,        // URI исходного документа
    pub document_hash: String,
    pub document_title: String, // Заголовок документа
    pub document_number: String,    // Номер документа
    pub document_sign_date: Date, // Дата подписания документа
    pub has_embeddings: bool,
    pub chunks_count: u32,
    pub chunks: Vec<Chunk>,
    pub collection_id: uuid::Uuid,
    pub status: DocumentStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Collection
{
    pub id: Uuid,        
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
}
impl ToString for Collection
{
    fn to_string(&self) -> String 
    {
        self.description.clone()
    }
}
impl AsRef<str> for Collection
{
    fn as_ref(&self) -> &str 
    {
        &self.description
    }
}


impl From<CollectionDbo> for Collection
{
    fn from(value: CollectionDbo) -> Self 
    {
        Self 
        { 
            id: value.id,
            name: value.name,
            description: value.description,
            keywords: value.keywords,
        }   
    }
}
impl From<Collection> for CollectionDbo
{
    fn from(value: Collection) -> Self 
    {
        Self 
        { 
            id: value.id,
            name: value.name,
            description: value.description,
            keywords: value.keywords,
        }   
    }
}

impl From<DocumentDbo> for Document
{
    fn from(value: DocumentDbo) -> Self 
    {
        Self 
        { 
            document_uri: value.document_uri,
            document_hash: value.document_hash,
            document_title: value.document_title,
            document_number: value.document_number,
            document_sign_date: value.document_sign_date,
            has_embeddings: value.has_embeddings,
            chunks_count: value.chunks_count,
            collection_id: value.collection_id,
            chunks: value.chunks,
            status: value.status
        }   
    }
}

impl From<DocumentCardDbo> for Document
{
    fn from(value: DocumentCardDbo) -> Self 
    {
        Self 
        { 
            document_uri: value.document_uri,
            document_hash: value.document_hash,
            document_title: value.document_title,
            document_number: value.document_number,
            document_sign_date: value.document_sign_date,
            has_embeddings: value.has_embeddings,
            chunks_count: value.chunks_count,
            chunks: Vec::with_capacity(0),
            collection_id: value.collection_id,
            status: value.status
        }      
    }
}

impl Document
{
    //Получаем None если чанков нет, иначе создаем документ на основе первого чанка и дополняем его метаданными
    pub fn from_chunks(chunks: Vec<Chunk>, collection_id: Uuid) -> Option<Self>
    {
        let first_chunk = chunks.first()?.clone();
        Some(Self 
        { 
            document_uri: first_chunk.document_url,
            document_hash: first_chunk.hash,
            document_title: first_chunk.title,
            document_number: first_chunk.number,
            document_sign_date: first_chunk.sign_date,
            has_embeddings: false,
            chunks_count: chunks.len() as u32,
            chunks,
            collection_id,
            status: DocumentStatus::NotLoaded,
            
        })
    }
}

///Облегченная структура для фроненда, без чанков
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentCard
{
    pub document_uri: String,
    pub document_hash: String,
    pub document_title: String,
    pub document_number: String, 
    pub document_sign_date: Date,
    pub has_embeddings: bool,
    pub chunks_count: u32,
    pub collection_id: Uuid,
    pub status: DocumentStatus,
}

impl From<DocumentDbo> for DocumentCard
{
    fn from(value: DocumentDbo) -> Self 
    {
        Self 
        { 
            document_uri: value.document_uri,
            document_hash: value.document_hash,
            document_title: value.document_title,
            document_number: value.document_number,
            document_sign_date: value.document_sign_date,
            has_embeddings: value.has_embeddings,
            chunks_count: value.chunks_count,
            collection_id: value.collection_id,
            status: value.status
        }   
    }
}
impl From<Document> for DocumentCard
{
    fn from(value: Document) -> Self 
    {
        Self 
        { 
            document_uri: value.document_uri,
            document_hash: value.document_hash,
            document_title: value.document_title,
            document_number: value.document_number,
            document_sign_date: value.document_sign_date,
            has_embeddings: value.has_embeddings,
            chunks_count: value.chunks_count,
            collection_id: value.collection_id,
            status: value.status
        }      
    }
}

impl From<DocumentCardDbo> for DocumentCard
{
    fn from(value: DocumentCardDbo) -> Self 
    {
        Self 
        { 
            document_uri: value.document_uri,
            document_hash: value.document_hash,
            document_title: value.document_title,
            document_number: value.document_number,
            document_sign_date: value.document_sign_date,
            has_embeddings: value.has_embeddings,
            chunks_count: value.chunks_count,
            status: value.status,
            collection_id: value.collection_id,
        }      
    }
}

