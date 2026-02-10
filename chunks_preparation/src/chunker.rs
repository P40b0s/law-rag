use rag_core::{Chunk, Encoder, ServiceStatus};
use tokio::sync::mpsc::UnboundedSender;

pub trait Chunker
{
    fn get_chunks<'m, E: Encoder + Sync + Send>(&self, encoder: &E, sender: UnboundedSender<ServiceStatus>) -> impl std::future::Future<Output = anyhow::Result<Vec<Chunk>>> + Send;
}