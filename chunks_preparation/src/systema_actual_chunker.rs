use std::fmt::Debug;

use rag_core::{Chunk, ChunkMeta, Encoder, ServiceStatus};
use systema_client::DocumentNodes;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{debug, info, warn};
use utilites::Date;

use crate::{chunker::Chunker, splitter::Splitter};

pub struct SystemaActualChunker<C: ToString + Debug + AsRef<str>>(DocumentNodes<C>);

impl<C: ToString + Debug + AsRef<str>> SystemaActualChunker<C>
{
    pub fn new(nodes: DocumentNodes<C>) -> Self
    {
        Self(nodes)
    }
}

impl<C: ToString + Debug + AsRef<str>> Chunker for SystemaActualChunker<C>
{
    async fn get_chunks<'m, E: Encoder>(&self, encoder: &E, sender: UnboundedSender<ServiceStatus>) -> anyhow::Result<Vec<Chunk>>
    {
        let hash = self.0.hash();
        let count = self.0.node_count();
        //если будем разбивать на буольшее количество чанков чем один абзац - один чанк то необходимо поправить
        let mut chunks = Vec::with_capacity(count);
        info!("Ноды документы были успешно получены: {} шт.", count);
        let chunker = Splitter::new(encoder).await?;
        let mut current = 1;
        let mut node_index = 0;
        for node in &self.0
        {
            node_index += 1;
            info!("Processing node {}/{}, current chunks: {}", node_index, count, current - 1);

            //бьем текст на куски тут и для каждого создаем чанку
            let content = node.converted_content().as_ref();
            info!("Node {}: content length {} bytes", node_index, content.len());

            let splitted = chunker.split_text(content).await?;

            info!("Node {}: split into {} chunks", node_index, splitted.len());

            for (text_idx, text) in splitted.into_iter().enumerate()
            {
                debug!("Creating chunk {} from node {} (text_idx: {})", current, node_index, text_idx);

                debug!("Finding parents path for chunk {}", current);
                let path_start = std::time::Instant::now();
                let path = self.0.find_all_parents_as_str(&node);
                let path_duration = path_start.elapsed();
                debug!("Parents path found: {} chars in {:?}", path.len(), path_duration);

                if path_duration.as_secs() > 5 {
                    warn!("Slow find_all_parents_as_str for chunk {}: took {:?}", current, path_duration);
                }

                let chunk = Chunk
                {
                    publication_url: self.0.publication_url().to_owned(),
                    document_url: format!("http://actual.pravo.gov.ru/list.html#hash={}", self.0.hash()),
                    title: self.0.title().to_owned(),
                    number: self.0.number().to_owned(),
                    sign_date: self.0.sign_date().to_owned(),
                    hash: self.0.hash().to_owned(),
                    path,
                    links_hashes: node.links_hashes().cloned(),
                    content: text.content,
                    embeddings: None,
                    meta: Some(ChunkMeta
                    {
                        chunk_index: text.chunk_index,
                        token_count: text.token_count
                    })
                };

                debug!("Sending status for chunk {}", current);
                let _ = sender.send(ServiceStatus::process_chunk(&hash, current, count));

                debug!("Pushing chunk {} to vector", current);
                chunks.push(chunk);
                current +=1;
                debug!("chunk {}/{} created", current, count);
            }
            info!("Node {}/{} completed, total chunks created: {}", node_index, count, current - 1);
        }
        info!("All nodes processed successfully, total chunks: {}", chunks.len());
        Ok(chunks)
    }
}