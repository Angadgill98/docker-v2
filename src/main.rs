use tokio::sync::oneshot;


mod server;
mod error;
mod controller;
mod manager;
mod IP_pool;
mod cli;
mod client;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let (sender,reciver)=oneshot::channel::<bool>();
    tokio::spawn(async move{
        server::Init(sender).await;

    });
    
    match reciver.await {
        Ok(value) => {
            println!("Server running");
        }
        Err(_) => {
            println!("Server dropped the sender");
            return;
        }
    }
    

    let mut client =
        match client::Client::init().await {

            Ok(client) => client,

            Err(e) => {
                eprintln!(
                    "Client Error: {:?}",
                    e
                );
                return;
            }
        };

    cli::run(&mut client).await;
}
