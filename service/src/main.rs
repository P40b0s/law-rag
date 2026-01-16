mod server;
mod state;
mod error;
mod configuration;
mod types;
//mod static_files;
pub use error::Error;
pub mod services;
mod api;

#[tokio::main]
async fn main() 
{
    let _ = logger::StructLogger::new_custom(logger::LevelFilter::Info, None);
    let _ = server::start().await;
}
