use rag_core::{Chunk, Encoder, ServiceStatus};
use tokio::sync::mpsc::UnboundedSender;

pub trait Chunker
{
    async fn get_chunks<'m, E: Encoder>(&self, encoder: &E, sender: UnboundedSender<ServiceStatus>) -> anyhow::Result<Vec<Chunk>>;
}