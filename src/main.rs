use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::{thread, time};

use simple_websockets::{Event, Message, Responder};

mod buffer;

enum ClientData {
    Painter,
    Canvas,
    Unknown,
}

struct Client {
    data: ClientData,
    responder: Responder,
    name: String,
    url: String,
}

const WHO_ARE_YOU: &str = "?";
const SEND_ME_PIXELS: &str = "p";

fn poll_painters(clients: Arc<RwLock<HashMap<u64, Client>>>) {
    loop {
        thread::sleep(time::Duration::from_secs(1));
        {
            let cs = clients.read().unwrap();
            for client in (*cs).values() {
                if let ClientData::Painter = &client.data {
                    client
                        .responder
                        .send(Message::Text(format!("{{\"msg\": \"{SEND_ME_PIXELS}\"}}")));
                }
            }
        }
    }
}

fn main() {
    let event_hub = simple_websockets::launch(8080).expect("failed to listen on port 8080");
    let clients: Arc<RwLock<HashMap<u64, Client>>> = Arc::new(RwLock::new(HashMap::new()));
    let clients_for_thread = Arc::clone(&clients);

    let mut image_buffer: crate::buffer::Buffer = crate::buffer::Buffer::new();

    thread::spawn(move || {
        poll_painters(clients_for_thread);
    });

    loop {
        match event_hub.poll_event() {
            Event::Connect(client_id, responder) => {
                println!("A client connected with id #{}", client_id);
                {
                    let mut cs = clients.write().unwrap();
                    cs.insert(
                        client_id,
                        Client {
                            data: ClientData::Unknown,
                            responder: responder.clone(),
                            name: Default::default(),
                            url: Default::default(),
                        },
                    );
                }
                responder.send(Message::Text(format!("{{\"msg\": \"{WHO_ARE_YOU}\"}}")));
            }
            Event::Disconnect(client_id) => {
                println!("Client #{} disconnected.", client_id);
                {
                    let mut cs = clients.write().unwrap();
                    cs.remove(&client_id);
                }
                image_buffer.remove(client_id);
            }
            Event::Message(client_id, message) => match message {
                Message::Binary(pixels) => {
                    image_buffer.update(client_id, pixels);
                }
                Message::Text(text) => {
                    let _ = jsonic::parse(&text).map(|sent| match sent["msg"].as_str() {
                        Some(WHO_ARE_YOU) => {
                            let mut cs = clients.write().unwrap();

                            if let Some(client) = cs.get(&client_id) {
                                client.responder.send(Message::Text(format!(
                                    "{{\"msg\": \"size\", \"w\": {}, \"h\": {}}}",
                                    buffer::BUFFER_PIXELS,
                                    buffer::BUFFER_PIXELS
                                )));
                            };

                            cs.entry(client_id).and_modify(|client| {
                                match sent[WHO_ARE_YOU].as_str() {
                                    Some("painter") => {
                                        client.data = ClientData::Painter;
                                        client.name =
                                            String::from(sent["name"].as_str().unwrap_or_default());
                                        client.url =
                                            String::from(sent["url"].as_str().unwrap_or_default());
                                        image_buffer.insert(client_id);
                                    }
                                    Some("canvas") => client.data = ClientData::Canvas,
                                    Some(_) | None => (),
                                }
                            });
                        }
                        Some(SEND_ME_PIXELS) => {
                            let cs = clients.read().unwrap();
                            if let Some(client) = cs.get(&client_id) {
                                let image = Vec::<u8>::from(&image_buffer);
                                if !image.is_empty() {
                                    let mut message =
                                        (image_buffer.dim() as u16).to_be_bytes().to_vec();
                                    message.extend(image);
                                    client.responder.send(Message::Binary(message));
                                }
                            }
                        }
                        Some(msg) => {
                            eprintln!("Received unknown message: {}", msg);
                        }
                        None => todo!(),
                    });
                }
            },
        }
    }
}
