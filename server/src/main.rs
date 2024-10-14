use server::Server;

mod server;
mod game;
mod connection;

#[tokio::main]
async fn main() {
    let _ = Server::init(vec![]).await;
}
