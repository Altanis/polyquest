use std::{collections::HashMap, net::SocketAddr};

use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, ConnectInfo, State},
    response::IntoResponse
};
use futures::{stream::SplitSink, SinkExt, StreamExt};
use shared::{connection::packets::ServerboundPackets, utils::codec::BinaryCodec};

use crate::{game::entity::base::Entity, server::{Server, ServerGuard, WrappedServer}};

use super::packets;

pub struct WebSocketClient {
    pub sender: SplitSink<WebSocket, Message>
}

pub struct WebSocketServer {
    clients: HashMap<u32, WebSocketClient>,
    ticks: u32
}

impl WebSocketServer {
    pub fn new() -> WebSocketServer {
        WebSocketServer {
            clients: HashMap::with_capacity(100),
            ticks: 0
        }
    }

    pub async fn handle_incoming_connection(
       socket: WebSocketUpgrade,
       ConnectInfo(_): ConnectInfo<SocketAddr>,
       State(server): State<WrappedServer>
    ) -> impl IntoResponse {
        socket.on_upgrade(move |socket| {
            WebSocketServer::accept_client(socket, server.clone())
        })
    }

    pub async fn accept_client(socket: WebSocket, server: WrappedServer) {
        let (mut receiver, id) = {
            let mut full_server = server.lock().await;
            let id = full_server.game_server.get_server().get_next_id();
    
            let (sender, receiver) = socket.split();

            full_server.ws_server.clients.insert(id, WebSocketClient { sender });
            full_server.game_server.get_server().insert_entity(Entity::from_id(id));

            (receiver, id)
        };

        while let Some(Ok(message)) = receiver.next().await {
            let mut full_server = server.lock().await;
            if let Err(ban) = WebSocketServer::handle_message(&mut full_server, message, id) {
                WebSocketServer::close_client(&mut full_server, id, ban);
            }
        }
    }

    pub fn handle_message(full_server: &mut ServerGuard, message: Message, id: u32) -> Result<(), bool> {
        match message {
            Message::Binary(data) => {
                let mut codec = BinaryCodec::from_bytes(data);
                let header: ServerboundPackets = (codec.decode_varuint().ok_or(true)? as u8)
                    .try_into()
                    .map_err(|_| true)?;

                match header {
                    ServerboundPackets::Spawn => packets::handle_spawn_packet(full_server, id, codec),
                    ServerboundPackets::Input => packets::handle_input_packet(full_server, id, codec),
                    ServerboundPackets::Stats => packets::handle_stats_packet(full_server, id, codec),
                    ServerboundPackets::Upgrade => packets::handle_upgrade_packet(full_server, id, codec)
                }
            },
            Message::Close(_) => Err(false),
            _ => Err(true)
        }
    }

    /// Closes the client.
    pub fn close_client(full_server: &mut ServerGuard, id: u32, ban: bool) {
        println!("Client # {} is being {}.", id, if ban { "banned" } else { "closed forcefully" });

        full_server.game_server.get_server().delete_entity(id);
        full_server.ws_server.clients.remove(&id);
    }

    pub async fn tick(full_server: &mut Server) {
        full_server.ws_server.ticks += 1;

        for (id, ws_client) in full_server.ws_server.clients.iter_mut() {
            let mut outgoing_packets = {
                let Some(mut entity) = full_server.game_server.get_server().get_entity(*id) else { continue; };
                let packets = entity.connection.outgoing_packets.clone();
                entity.connection.outgoing_packets.clear();

                packets
            };

            while let Some(packet) = outgoing_packets.pop() {
                let _ = ws_client.sender.send(Message::Binary(packet.out())).await;
            }
        }
    }
}