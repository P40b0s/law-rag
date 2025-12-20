use systema_client::{DocumentKindSearchParams, SystemaIpsApi};
use tracing::info;
use utilites::Date;

use crate::chunking::DocumentMetadata;

mod logger;
mod model;
mod embedding;
mod error;
mod chunking;
mod structure;
mod pipeline;
mod document;
mod document_handler;
mod chunks;
mod qdrant;

pub static TEST_DOC1: &str = include_str!("./test_doc1.html");
#[tokio::main]
async fn main()
{
    logger::init();
    let chunk = chunking::LegalDocumentChunker::new(model::LongContextModel::BgeReranker);
    let tokenizer = embedding::LongContextEmbedder::new(model::LongContextModel::BgeReranker).unwrap();
    let document = SystemaIpsApi::search(
            &[DocumentKindSearchParams::Fz, DocumentKindSearchParams::Fkz],
            "273-фз",
            Date::new_date(29, 12, 2012)).await.unwrap();
    let html = document.get_document_html().await.unwrap();
    let cc = chunk.chunk_document(html, &DocumentMetadata::new("www.w.ru".to_owned(), "TEST".to_owned()), tokenizer.get_tokenizer()).await.unwrap();
    info!("chunks: {:#?}", cc);
}

