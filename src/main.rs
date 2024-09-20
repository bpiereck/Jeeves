use std::cmp::Ordering;
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
    naughty: u32,
}

enum Action {
    RemoveClient,
    SendMessage(Message),
}

const WHO_ARE_YOU: &str = "?";
const SEND_ME_PIXELS: &str = "p";
const NAUGHTY_WARNING: u32 = 50;

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

fn handle_error(message: String, client: &mut Client) -> Action {
    client.naughty += 1;
    let json = format!(
        "{{\"msg\": \"error\", \"FINAL WARNING {}\", \"naughty\": {}}}",
        message.replace("\"", "\\\""),
        client.naughty
    );
    match client.naughty.cmp(&NAUGHTY_WARNING) {
        Ordering::Less => Action::SendMessage(Message::Text(json.replace("FINAL WARNING ", ""))),
        Ordering::Equal => Action::SendMessage(Message::Text(json)),
        Ordering::Greater => Action::RemoveClient,
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
                            naughty: 0,
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
                    if let Err(error) = image_buffer.update(client_id, pixels) {
                        match error {
                            crate::buffer::UpdateError::Server(message) => {
                                eprintln!("Error updating pixels for {}: {}", client_id, message);
                            },
                            crate::buffer::UpdateError::Client(message) => {
                                let mut cs = clients.write().unwrap();
                                if let Some(client) = cs.get_mut(&client_id) {
                                    match handle_error(message, client) {
                                        Action::RemoveClient => {
                                            client.responder.close();
                                            cs.remove(&client_id);
                                            image_buffer.remove(client_id);
                                        },
                                        Action::SendMessage(msg) => {
                                            client.responder.send(msg);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Message::Text(text) => {
                    let _ = jsonic::parse(&text)
                        .map_err(|e| {
                            let mut cs = clients.write().unwrap();
                            if let Some(client) = cs.get_mut(&client_id) {
                                match handle_error(format!("(Cannot parse) {}", e), client) {
                                    Action::RemoveClient => {
                                        client.responder.close();
                                        cs.remove(&client_id);
                                        image_buffer.remove(client_id);
                                    },
                                    Action::SendMessage(msg) => {
                                        client.responder.send(msg);
                                    }
                                }
                            } else {
                                eprintln!("Unknown client id {}", client_id);
                            }
                        })
                        .map(|sent| {
                            let mut cs = clients.write().unwrap();
                            if let Some(client) = cs.get_mut(&client_id) {
                                match sent["msg"].as_str() {
                                    Some(WHO_ARE_YOU) => {
                                        let size_message = Message::Text(format!(
                                            "{{\"msg\": \"size\", \"w\": {}, \"h\": {}}}",
                                            buffer::BUFFER_PIXELS,
                                            buffer::BUFFER_PIXELS
                                        ));

                                        match sent[WHO_ARE_YOU].as_str() {
                                            Some("painter") => {
                                                client.data = ClientData::Painter;
                                                client.name =
                                                    String::from(sent["name"].as_str().unwrap_or_default());
                                                client.url =
                                                    String::from(sent["url"].as_str().unwrap_or_default());
                                                if let Err(error) = image_buffer.insert(client_id) {
                                                    eprintln!("{}", error);
                                                    client.responder.close();
                                                    cs.remove(&client_id);
                                                } else {
                                                    client.responder.send(size_message);
                                                }
                                            },
                                            Some("canvas") => {
                                                client.data = ClientData::Canvas;
                                                client.responder.send(size_message);
                                            },
                                            Some(who) => {
                                                match handle_error(format!("{} is not a valid ?. Should be painter or canvas", who), client) {
                                                    Action::RemoveClient => {
                                                        client.responder.close();
                                                        cs.remove(&client_id);
                                                        image_buffer.remove(client_id);
                                                    },
                                                    Action::SendMessage(msg) => {
                                                        client.responder.send(msg);
                                                    }
                                                }
                                            },
                                            None => {
                                                match handle_error(String::from("Expected field ?"), client) {
                                                    Action::RemoveClient => {
                                                        client.responder.close();
                                                        cs.remove(&client_id);
                                                        image_buffer.remove(client_id);
                                                    },
                                                    Action::SendMessage(msg) => {
                                                        client.responder.send(msg);
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    Some(SEND_ME_PIXELS) => {
                                        let image = <&Vec::<u8>>::from(&image_buffer);
                                        if !image.is_empty() {
                                            let mut message =
                                                (image_buffer.dim() as u16).to_be_bytes().to_vec();
                                            message.extend(image);
                                            client.responder.send(Message::Binary(message));
                                        }
                                    },
                                    Some(msg) => {
                                        match handle_error(format!("Unknown message: {}", msg), client) {
                                            Action::RemoveClient => {
                                                client.responder.close();
                                                cs.remove(&client_id);
                                                image_buffer.remove(client_id);
                                            },
                                            Action::SendMessage(msg) => {
                                                client.responder.send(msg);
                                            }
                                        }
                                    },
                                    None => {
                                        match handle_error(String::from("Invalid message"), client) {
                                            Action::RemoveClient => {
                                                client.responder.close();
                                                cs.remove(&client_id);
                                                image_buffer.remove(client_id);
                                            },
                                            Action::SendMessage(msg) => {
                                                client.responder.send(msg);
                                            }
                                        }
                                    }
                                }
                            }
                        });
                }
            },
        }
    }
}
