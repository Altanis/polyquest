use std::{net::SocketAddr, sync::Arc};
use axum::Router;
use tokio::sync::{Mutex as AsyncMutex, MutexGuard};
use crate::{connection::wss::WebSocketServer, game::state::{GameServer, GameState}};

pub type WrappedServer = Arc<AsyncMutex<Server>>;

pub const PORT: u16 = 8080;
pub const FPS: u64 = 25;
pub const MSPT: u64 = 1000 / FPS;
pub const CELL_SIZE: u32 = 8;

pub struct Server {
    pub game_server: GameServer,
    pub ws_server: WebSocketServer
}

pub type ServerGuard<'a> = MutexGuard<'a, Server>;

impl Server {
    /// Initializes the server.
    pub async fn init(game_states: Vec<GameState>) {
        let wrapped_server = Arc::new(AsyncMutex::new(Server {
            game_server: GameServer::new(game_states),
            ws_server: WebSocketServer::new()
        }));

        let wrapped_server_clone = wrapped_server.clone();
        let wrapped_server_clone_2 = wrapped_server.clone();

        tokio::task::spawn(async move {
            let router = Router::new()
                .route("/ws", axum::routing::get(WebSocketServer::handle_incoming_connection))
                .with_state(wrapped_server_clone);

            let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", PORT))
                .await
                .unwrap();

            axum::serve(
                listener,
                router.into_make_service_with_connect_info::<SocketAddr>()
            )
                .await
                .unwrap();
        });

        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(MSPT));
        loop {
            interval.tick().await;
            wrapped_server_clone_2.lock().await.tick().await;
        }
    }

    /// Ticks the server.
    pub async fn tick(&mut self) {
        self.game_server.tick();
        WebSocketServer::tick(self).await;
    }
}