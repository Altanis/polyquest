use gloo::console::console;
use shared::{connection::packets::ClientboundPackets, utils::codec::BinaryCodec};
use wasm_bindgen_futures::spawn_local;
use web_sys::{js_sys::{ArrayBuffer, Uint8Array}, wasm_bindgen::{prelude::Closure, JsCast}, BinaryType, Event, MessageEvent, Performance, WebSocket};

use crate::world::{get_world, World};

use super::packets::{handle_notification_packet, handle_update_packet};

const IS_PROD: bool = false;
const URL: &str = if IS_PROD {
    "ws://108.29.192.90:8080/ws"
} else {
    "ws://127.0.0.1:8080/ws"
};

const MAX_RETRIES: usize = 3;

#[derive(PartialEq)]
pub enum ConnectionState {
    Connected,
    Connecting,
    Failed
}

pub struct Connection {
    pub state: ConnectionState,
    pub latency: f32,
    pub mspt: f32,

    retries: usize,
    last_ping: f64,
    
    socket: WebSocket
}

impl Connection {
    pub fn new() -> Connection {
        let socket = WebSocket::new(URL)
            .expect("ws api not supported");
        socket.set_binary_type(BinaryType::Arraybuffer);

        let connection = Connection { 
            state: ConnectionState::Connecting,
            latency: 0.0,
            mspt: 0.0,
            retries: 0,
            last_ping: 0.0,
            socket 
        };

        connection.setup_event_handlers();

        connection
    }

    fn setup_event_handlers(&self) {
        let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
            spawn_local(async move {
                let mut world = get_world();

                if let Ok(buffer) = event.data().dyn_into::<ArrayBuffer>() {
                    let array = Uint8Array::new(&buffer);
                    let mut data = vec![0; array.length() as usize];
                    array.copy_to(&mut data);
    
                    let codec = BinaryCodec::from_bytes(data);
                    Connection::on_message(&mut world, codec);
                }
            });
        }) as Box<dyn FnMut(MessageEvent)>);

        let onopen_callback = Closure::wrap(Box::new(move |_| {
            spawn_local(async {
                let mut world = get_world();
                Connection::on_open(&mut world);
            });
        }) as Box<dyn FnMut(Event)>);

        let onerror_callback = Closure::wrap(Box::new(move |_| {
            spawn_local(async {
                let mut world = get_world();
                Connection::on_error(&mut world);
            });
        }) as Box<dyn FnMut(Event)>);

        let onclose_callback = Closure::wrap(Box::new(move |_| {
            spawn_local(async {
                let world = get_world();
            });
        }) as Box<dyn FnMut(Event)>);

        self.socket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        self.socket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        self.socket.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        self.socket.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));

        onmessage_callback.forget();
        onopen_callback.forget();
        onerror_callback.forget();
        onclose_callback.forget();
    }

    fn on_open(world: &mut World) {
        let connection = &mut world.connection;
        connection.retries = 0;
        connection.state = ConnectionState::Connected;
    }

    fn on_error(world: &mut World) {
        let connection = &mut world.connection;
        connection.retries += 1;

        if connection.retries <= MAX_RETRIES {
            connection.socket = WebSocket::new(URL)
                .expect("ws api not supported");

            connection.setup_event_handlers();
        } else {
            connection.state = ConnectionState::Failed;
        }
    }

    fn on_close(world: &mut World) {

    }

    fn on_message(world: &mut World, mut codec: BinaryCodec) {
        let header: ClientboundPackets = (codec.decode_varuint().unwrap() as u8).try_into().unwrap();
        match header {
            ClientboundPackets::Update => handle_update_packet(world, codec),
            ClientboundPackets::Notifications => handle_notification_packet(world, codec)
        }
    }

    pub fn send_message(&self, data: BinaryCodec) {
        let _ = self.socket.send_with_u8_array(data.out().as_slice());
    }
}