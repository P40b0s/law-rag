use std::{convert::Infallible, sync::Arc};
use database::ChunkProcess;
use serde::Serialize;
use futures_util::stream::Stream;
use axum::{extract::State, response::sse::{Event, KeepAlive, Sse}};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::{state::AppState};

pub struct SSEService(Arc<Broadcaster>);

impl SSEService
{
    pub fn new() -> Self
    {
        Self(Broadcaster::new())
    }
    pub fn send_command(&self, command: SSECommand)
    {
        self.0.send_command(command);
    }
    pub fn add_client(&self) -> broadcast::Receiver<Event> 
    {
        self.0.fanout.subscribe()
    }

    pub fn load_chunk_process(&self, process: ChunkProcess)
    {
        let command = SSECommand::LoadChunk { doc_uri: process.uri.clone(), process };
        self.0.send_command(command);
    }
}

#[derive(Serialize, Debug, Clone)]
//#[serde(tag = "command", content = "content")]
#[serde(tag = "event", content = "content")]
#[serde(rename_all="snake_case")]
pub enum SSECommand
{
    LoadChunk { doc_uri: String, process: ChunkProcess },
}


pub struct Broadcaster 
{
    fanout: broadcast::Sender<Event>,
}
impl Broadcaster 
{
    pub fn new() -> Arc<Self> 
    {
        let (tx, _) = broadcast::channel(16);
        Arc::new(Broadcaster { fanout: tx })
    }
    pub fn add_client(&self) -> broadcast::Receiver<Event> 
    {
        self.fanout.subscribe()
    }
    pub fn send_message(&self, msg: &str)
    {
        self.broadcast(Event::default().data(msg));
    }
    pub fn send_named_message(&self, event_name: &str, msg: &str)
    {
        self.broadcast(Event::default().data(msg).event(event_name));
    }
    pub fn send_named_object<O: Serialize>(&self, event_name: &str, obj: &O)
    {
        let msg = serde_json::to_string(&obj).unwrap();
        self.broadcast(Event::default().data(msg).event(event_name));
    }
    pub fn send_object<O: Serialize>(&self, obj: &O)
    {
        let msg = serde_json::to_string(&obj).unwrap();
        self.broadcast(Event::default().data(msg));
    }
    //странно но так не работает....
    //все же так работает, надо было просто добавить со стороны клиента 
    // evtSource.addEventListener("command", (c)=>
    // {
    //     console.log(`event:command: ${c.data}`);
    // })
    pub fn send_command(&self, command: SSECommand)
    {
        let msg = serde_json::to_string(&command).unwrap();
        self.broadcast(Event::default().data(msg).event("command"));
    }
    pub fn broadcast(&self, event: Event)
    {
        if let Err(e) = self.fanout.send(event)
        {
            logger::warn!("Ошибка отправки сообщения, нет ни одного подключенного клиента {}", e.to_string())
        }
    }
}

///we cannot give handler self arg, therefore we add a client by calling the handler
pub async fn sse_handler(State(app_state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> 
{
    let mut rx = app_state.services.sse_service.add_client();
    let stream = async_stream::stream! 
    {
        yield Ok(Event::default().data("Вы подключены к шине сообщений"));
        loop 
        {
            let msg = rx.recv().await.unwrap();
            yield Ok(msg);
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}