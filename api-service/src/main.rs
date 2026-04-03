mod server;
mod state;
mod error;
mod configuration;
mod types;
//mod static_files;
pub use error::Error;
use tracing::info;
pub mod services;
mod api;
mod logger;

#[tokio::main]
async fn main() 
{
    let _ = logger::init();
    let _ = server::start().await;
    info!("Программа завершена!");
}
