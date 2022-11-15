extern crate common;

use std::io::Write;
use common::message::{Message, MessageType};
use log::{debug, LevelFilter};
use simplelog::{SimpleLogger, Config};
use common::crypt;

fn main() {
    SimpleLogger::init(LevelFilter::Debug, Config::default()).unwrap();
    // create two key pairs
    let private_key1 = crypt::create_private_key();
    let public_key1 = crypt::create_public_key(private_key1.clone());
    let private_key2 = crypt::create_private_key();
    let public_key2 = crypt::create_public_key(private_key2.clone());

    // create shared key from the two key pairs
    let shared_key1 = crypt::create_shared_key(private_key1, public_key2);
    let shared_key2 = crypt::create_shared_key(private_key2, public_key1);

    // verify that the shared keys are the same
    if shared_key1 == shared_key2 {
        debug!("Shared keys are the same");
    } else {
        debug!("Shared keys are not the same");
    }

    let original_message = "Hello, world!".to_string();

    // encrypt the message
    let encrypted_message = crypt::encrypt_data(original_message.clone().as_bytes().to_vec(), shared_key1.clone());
    let decrypted_message = crypt::decrypt_data(encrypted_message.clone(), shared_key2.clone());

    // print the message
    debug!("Original message: {}", original_message);
    debug!("Encrypted message: {:?}", encrypted_message);
    debug!("Decrypted message: {:?}", decrypted_message);
    debug!("Decrypted message: {}", String::from_utf8(decrypted_message.clone()).unwrap());

    // check if the message is the same
    if original_message == String::from_utf8(decrypted_message.clone()).unwrap() {
        debug!("Messages are the same");
    } else {
        debug!("Messages are not the same");
    }

    // create a message of random bytes
    let mut random_message = String::new();
    for _ in 0..8192 {
        let random_number = rand::random::<u8>();
        random_message.push(random_number as char);
    }

    // encrypt the message
    let encrypted_message = crypt::encrypt_data(random_message.clone().as_bytes().to_vec(), shared_key1.clone());
    let decrypted_message = crypt::decrypt_data(encrypted_message.clone(), shared_key2.clone());

    debug!("test 2");

    // check if the message is the same
    if random_message == String::from_utf8(decrypted_message.clone()).unwrap() {
        debug!("Messages are the same");
    } else {
        debug!("Messages are not the same");
    }
}