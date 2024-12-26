#![feature(duration_millis_float)]
#![feature(let_chains)]
#![feature(generic_arg_infer)]

use game::state::GameState;
use server::Server;

mod server;
mod game;
mod connection;

#[tokio::main]
async fn main() {
    let _ = Server::init(vec![
        GameState::default()
    ]).await;
}
