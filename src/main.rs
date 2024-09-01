use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::{thread, time};

use simple_websockets::{Event, Message, Responder};

enum ClientData {
    Painter(Vec<u8>),
    Player,
    Canvas,
    Unknown,
}

struct Client {
    data: ClientData,
    responder: Responder,
}

const WHO_ARE_YOU: &str = "?";
const SEND_ME_PIXELS: &str = "p";
const CANVAS_WIDTH: usize = 20;
const CANVAS_HEIGHT: usize = 20;
const CANVAS_BUFFER_SIZE: usize = CANVAS_WIDTH * CANVAS_HEIGHT * 4; // 4 bytes / pixel
const CANVAS_SIZE: &str = "{\"msg\": \"size\", \"w\": 20, \"h\": 20}";

fn poll_painters(clients: Arc<RwLock<HashMap<u64, Client>>>) {
    loop {
        thread::sleep(time::Duration::from_secs(1));
        {
            let cs = clients.read().unwrap();
            for (id, client) in &*cs {
                if let ClientData::Painter(_) = &client.data {
                    println!("Asking for pixeldata from client: {}", id);
                    client
                        .responder
                        .send(Message::Text(String::from(SEND_ME_PIXELS)));
                }
            }
        }
    }
}

fn main() {
    let event_hub = simple_websockets::launch(8080).expect("failed to listen on port 8080");
    let clients: Arc<RwLock<HashMap<u64, Client>>> = Arc::new(RwLock::new(HashMap::new()));
    let clients_for_thread = Arc::clone(&clients);

    thread::spawn(move || {
        poll_painters(clients_for_thread);
    });

    loop {
        match event_hub.poll_event() {
            Event::Connect(client_id, responder) => {
                println!("A client connected with id #{}", client_id);
                let mut cs = clients.write().unwrap();
                cs.insert(
                    client_id,
                    Client {
                        data: ClientData::Unknown,
                        responder: responder.clone(),
                    },
                );
                responder.send(Message::Text(String::from(WHO_ARE_YOU)));
            }
            Event::Disconnect(client_id) => {
                println!("Client #{} disconnected.", client_id);
                let mut cs = clients.write().unwrap();
                cs.remove(&client_id);
            }
            Event::Message(client_id, message) => match message {
                Message::Binary(pixels) => {
                    let mut cs = clients.write().unwrap();
                    cs.entry(client_id).and_modify(|client| {
                        if let ClientData::Painter(ref mut pixeldata) = &mut client.data {
                            *pixeldata = pixels;
                        }
                    });
                }
                Message::Text(text) => {
                    let _ = jsonic::parse(&text).map(|sent| match sent["msg"].as_str() {
                        Some(WHO_ARE_YOU) => {
                            let mut cs = clients.write().unwrap();

                            if sent[WHO_ARE_YOU].as_str() != Some("player") {
                                if let Some(client) = cs.get(&client_id) {
                                    client
                                        .responder
                                        .send(Message::Text(String::from(CANVAS_SIZE)));
                                }
                            }

                            cs.entry(client_id).and_modify(|client| {
                                match sent[WHO_ARE_YOU].as_str() {
                                    Some("painter") => {
                                        client.data =
                                            ClientData::Painter(vec![0; CANVAS_BUFFER_SIZE])
                                    }
                                    Some("player") => client.data = ClientData::Player,
                                    Some("canvas") => client.data = ClientData::Canvas,
                                    Some(_) | None => (),
                                }
                            });
                        }
                        Some(SEND_ME_PIXELS) => {
                            let cs = clients.read().unwrap();
                            let count: u16 = cs
                                .iter()
                                .filter(|(_, client)| {
                                    matches!(&client.data, ClientData::Painter(_))
                                })
                                .count()
                                .try_into()
                                .expect("Too many connected painters!");
                            let mut pixels = cs
                                .iter()
                                .filter_map(|(id, client)| -> Option<Vec<u8>> {
                                    if let ClientData::Painter(pixeldata) = &client.data {
                                        let mut data = id.to_be_bytes().to_vec();
                                        data.append(&mut (pixeldata.clone()));
                                        Some(data)
                                    } else {
                                        None
                                    }
                                })
                                .flatten()
                                .collect::<Vec<u8>>();
                            let mut data = count.to_be_bytes().to_vec();
                            data.append(&mut pixels);
                            if let Some(client) = cs.get(&client_id) {
                                client.responder.send(Message::Binary(data));
                            };
                        }
                        Some(_) => todo!(),
                        None => todo!(),
                    });
                }
            },
        }
    }
}
