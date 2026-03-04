use tracing::{Level, subscriber::set_global_default};

pub fn init() -> bool 
{
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_line_number(true)
        .with_file(true);
    match set_global_default(subscriber.finish()) 
    {
        Ok(_) => 
        {
            // Initialization was successful
            true
        }
        Err(_) => 
        {
            // A global default subscriber was already set
            false
        }
    }
}