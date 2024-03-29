use std::{error::{Error, self}, io::{self, Write, BufReader}, net::SocketAddr, fs::File, path, thread};

use client::Client;
use common::{user::User, message::{self, MessagePayload, Payload}, id, crypt};
use futures::{channel::mpsc, future::InspectOk, StreamExt};
use log::debug;
use tokio_util::codec::{Framed, BytesCodec, FramedWrite, FramedRead};
use bytes::{Bytes, BytesMut};
use common::message::{Message, MessageType};
use tokio::net::{TcpListener, TcpStream};

use futures::SinkExt;
use std::env;
use std::sync::Arc;

use simplelog::*;

mod client;
extern crate common;

fn setup() -> User {
    // if file exists, read from file
    if path::Path::new("me.dat").exists() {
        let file = File::open("me.dat").unwrap();
        let reader = BufReader::new(file);
        // convert from bson to user
        let user: User = rmp_serde::from_read(reader).unwrap();
        return user;
    }

    // else, create new user
    // ask for the username
    let mut username = String::new();
    println!("Enter your username: ");
    io::stdin()
        .read_line(&mut username)
        .expect("Failed to read line");
    let username = username.trim().to_string();

    let user = User::new(username);

    // save the user to a file
    let mut file = std::fs::File::create("me.dat").unwrap();

    file.write_all(&user.to_bytes()).unwrap();

    user
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::init(LevelFilter::Debug, Config::default()).unwrap();

    // get the user either from a file or from the user
    let mut user = setup();

    let secret_key = crypt::create_private_key();

    let public_key = crypt::create_public_key(secret_key.clone());

    user.set_public_key(crypt::serialize_public_key(public_key));

    let mut client = Client::new(user.clone(), secret_key.clone());

    let stdin = FramedRead::new(tokio::io::stdin(), BytesCodec::new());
    let mut stdin = stdin.map(|i| i.map(BytesMut::freeze));
    let mut stdout = FramedWrite::new(tokio::io::stdout(), BytesCodec::new());

    // get address from args, or panic
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "".to_string());
    
    if addr == "" {
        log::error!("No address provided");
        // exit the program
        std::process::exit(1);
    }

    let mut stream = TcpStream::connect(addr).await?;
    let (reader, writer) = stream.split();
    let mut sink = FramedWrite::new(writer, BytesCodec::new());
    let mut stream = FramedRead::new(reader, BytesCodec::new());

    loop {
        tokio::select! {
            msg = stream.next() => {
                if let Some(Ok(msg)) = msg {
                    let raw: Vec<u8> = msg.to_vec();

                    let message = Message::from_bytes(raw.clone());

                    match message.message_type {
                        MessageType::Message => {
                            // load the payload
                            let mut payload = MessagePayload::from_bytes(message.payload);

                            // decrypt the message
                            payload.decrypt(client.get_shared_key());

                            // load the internal message from the payload
                            let text = String::from_utf8(payload.message).unwrap();

                            // print the message
                            /* format in HH:MM:SS */
                            let timestamp = id::to_formatted_timestamp(message.id, "%H:%M:%S");
                            println!("[{}] {}: {}", timestamp, payload.username, text);
                        },
                        MessageType::ConnectionReceive => {
                            let login_message = Message::new(MessageType::Login, user.clone().to_bytes());
                            sink.send(Bytes::from(login_message.to_bytes())).await?;
                            // create shared secret
                            let pub_key = crypt::deserialize_public_key(message.payload);
                            let shared_secret = crypt::create_shared_key(secret_key.clone(), pub_key);
                            client.set_shared_key(shared_secret);
                            debug!("Shared secret: {:?}", client.get_shared_key());
                        },
                        MessageType::Unknown => {
                            debug!("Received unknown message type");
                        },
                        _ => {
                            debug!("received a message of type {:?}", message.message_type);
                        }
                    }
                } else {
                    break
                }
            },
            input = stdin.next() => {
                if let Some(Ok(input)) = input {
                    let input = String::from_utf8(input.to_vec()).unwrap();
                    // remove the newline
                    let input = input.trim().to_string();

                    // split string into byte chunks
                    let chunks = input.as_bytes().chunks(256);

                    for chunk in chunks {
                        let chunk = crypt::encrypt_data(chunk.to_vec(), client.get_shared_key());
                        // TODO: Channels
                        let message_payload = MessagePayload::new(user.clone().username, "#general".to_string(), chunk);
                        let message = Message::new(MessageType::Message, message_payload.to_bytes());
                        sink.send(Bytes::from(message.to_bytes())).await?;
                    }
                } else {
                    break;
                }
            }
            // on close
            else => {
                break;
            }
        }
    }

    log::error!("Connection closed");
    // exit the program
    std::process::exit(1);

    Ok(())
}