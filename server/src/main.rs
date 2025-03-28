#![feature(duration_millis_float)]
#![feature(let_chains)]
#![feature(generic_arg_infer)]

use game::{physics::shg::SpatialHashGrid, state::GameState};
use server::{Server, CELL_SIZE};

mod server;
mod game;
mod connection;

#[tokio::main]
async fn main() {
    let _ = Server::init(vec![
        GameState {
            shg: SpatialHashGrid::new(256, CELL_SIZE),
            desired_orb_count: 300,
            ..GameState::default()
        }
    ]).await;
}
