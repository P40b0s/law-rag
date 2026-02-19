use rag_core::Converter;
use serde::{Deserialize, Serialize};
use scraper::Node;
use tracing::{debug, info};
use utilites::Date;
pub struct SystemaActualConverter;
pub enum Actions
{
    Superscript,
    Subscript,
    //ссылок пока нет
    //Link,
    Skip,
    None
}
impl Converter<String> for SystemaActualConverter
{
    fn convert(&self, html: String) -> String 
    {
        let cur_html = scraper::Html::parse_document(&html);
        let mut result_text = String::new();
        let mut actions = Actions::None;
        for node in  cur_html.tree.values()
        {
            match node
            {
                Node::Element(e) => 
                {
                    if e.name() == "span"
                    {
                        if e.has_class("W9", scraper::CaseSensitivity::AsciiCaseInsensitive)
                        {
                            actions = Actions::Superscript;
                        }
                        if e.has_class("W8", scraper::CaseSensitivity::AsciiCaseInsensitive)
                        {
                            actions = Actions::Subscript;
                        }
                        if e.has_class("mark", scraper::CaseSensitivity::AsciiCaseInsensitive)
                            || e.has_class("markx", scraper::CaseSensitivity::AsciiCaseInsensitive)
                            //|| e.has_class("cmd", scraper::CaseSensitivity::AsciiCaseInsensitive)
                        {
                            actions = Actions::Skip;
                        }
                        // if e.has_class("cmd", scraper::CaseSensitivity::AsciiCaseInsensitive)
                        // {

                        //     actions = Actions::Link;
                        //     <span class="cmd" cmdprm="gohash=b113c2e08341853ef53a8dad4585b513d96f85e0f3d0d246a25ecf52e40608db goparaid=0 goback=0">
                        // }
                    }
                    //debug!("found element: {:#?}", e);
                }
                Node::Text(t) =>
                {
                    match actions
                    {
                        Actions::Superscript =>
                        {
                            debug!("found text: {:#?}", t);
                            result_text.push_str(&["^", t].concat());
                            actions = Actions::None;
                        }
                        Actions::Subscript =>
                        {
                            debug!("found text: {:#?}", t);
                            result_text.push_str(&["_", t].concat());
                            actions = Actions::None;
                        }
                        Actions::Skip => { actions = Actions::None }
                        Actions::None =>
                        {
                            result_text.push_str(&t.replace("\u{a0}", " "));
                        }
                    }
                }
                _ => {}
            }
        }
        debug!("result text: {:#?}", result_text);
        result_text
    }
}

//<p id="p165"><span class="edx">12.&nbsp;Установить, что в случае, если по состоянию на 1&nbsp;мая 2026&nbsp;года сумма задолженности физического лица, указанная в частях 1&nbsp;и 2&nbsp;настоящей статьи (за исключением задолженности, не учитываемой в совокупной обязанности в соответствии с&nbsp;подпунктом 2 пункта 7 статьи 11<span class="W9">3</span> <span class="cmd-hide" cmdprm="gohash=b113c2e08341853ef53a8dad4585b513d96f85e0f3d0d246a25ecf52e40608db goparaid=0 goback=0">Налогового кодекса Российской Федерации</span>), в совокупности не превышает 500&nbsp;рублей и (или) сумма задолженности, не учитываемая в составе его совокупной обязанности в соответствии с подпунктом 2 пункта 7 статьи 11<span class="W9">3</span> Налогового кодекса Российской Федерации, не превышает 10&nbsp;000&nbsp;рублей, такая задолженность признается безнадежной к взысканию и подлежит списанию в размере, не погашенном на дату вынесения решения о признании задолженности безнадежной к взысканию и ее списании. Информационные сообщения, предусмотренные частью 3&nbsp;настоящей статьи, в отношении данных сумм задолженности и суммы задолженности, не учитываемой в совокупной обязанности физического лица в соответствии с&nbsp;подпунктом 2 пункта 7 статьи 11<span class="W9">3</span> <span class="cmd-hide" cmdprm="gohash=b113c2e08341853ef53a8dad4585b513d96f85e0f3d0d246a25ecf52e40608db goparaid=0 goback=0">Налогового кодекса Российской Федерации</span>, не направляются.</span><span class="markx">&nbsp;(Дополнение частью - Федеральный закон <span class="cmd-hide" cmdprm="gohash=9ba1e79973a0348999e09789280f0546258a12e408a60a09c52254290233fbcf goparaid=p1787 goback=1">от&nbsp;28.11.2025&nbsp;№&nbsp;425-ФЗ</span>)</span></p>


#[cfg(test)]
mod tests
{
    use std::sync::Arc;

    use rag_core::{Chunk, ChunkMeta, EmbeddingConfiguration};
    use systema_client::{DocumentNode, DocumentNodes};
    use tracing::{debug, info};
    use utilites::Date;
    use embedding::{RetriverModel};
    use crate::{chunker::Chunker, splitter::Splitter, systema_actual_chunker, systema_actual_converter::SystemaActualConverter};

    #[tokio::test]
    async fn test_converter()
    {
        rag_core::init();
        let converter = SystemaActualConverter{};
        let result = systema_client::SystemaClient::get_document_tree(
            Date::new_date(07, 06, 2025),
            "127-ФЗ",
            converter,
        )
        .await
        .unwrap();
        debug!("{:?}", &result);
        let emb_cfg = Arc::new(EmbeddingConfiguration::default());
        let model = RetriverModel::new(emb_cfg).await.unwrap();
        let chunker = systema_actual_chunker::SystemaActualChunker::new(result);
        let (sender, _receiver) = tokio::sync::mpsc::unbounded_channel();
        let chunks = chunker.get_chunks(&model, sender).await.unwrap();
        // for chunk in &chunks
        // {
        //     debug!("chunk created: {:#?}", chunk);
        // }
    }

     #[tokio::test]
    async fn test_nodes()
    {
        rag_core::init();
        let converter = SystemaActualConverter{};
        let result = 
            systema_client::SystemaClient::get_document(
                Date::new_date(31, 07, 2025),
                "287-ФЗ", converter).await.unwrap();
        debug!("{:?}", &result);
    }
}