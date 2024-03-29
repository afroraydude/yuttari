use crate::id::{create_id, IdType};
use serde::{Deserialize, Serialize};
use x25519_dalek::{PublicKey, StaticSecret};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub public_key: Vec<u8>,
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl User {
    pub fn new(username: String) -> User {
        let id = create_id(IdType::User);
        User { id, username, public_key: Vec::new() }
    }

    pub fn create_all(id: u64, username: String, public_key: Vec<u8>) -> User {
        User { id, username, public_key }
    }

    pub fn change_username(&mut self, username: String) {
        self.username = username;
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        rmp_serde::to_vec(self).unwrap()
    }

    pub fn from_bytes(raw: Vec<u8>) -> User {
        // try to deserialize the bson, if it fails, return an unknown user
        match rmp_serde::from_slice(&raw) {
            Ok(user) => user,
            Err(_) => User::new("Unknown".to_string()),
        }
    }

    pub fn deserialize_public_key(public_key: Vec<u8>) -> PublicKey {
        let mut public_key_bytes = [0u8; 32];
        for (i, byte) in public_key.iter().enumerate() {
            public_key_bytes[i] = *byte;
        }
        PublicKey::from(public_key_bytes)
    }

    pub fn set_public_key(&mut self, public_key: Vec<u8>) {
        self.public_key = public_key;
    }
}

impl Clone for User {
    fn clone(&self) -> Self {
        User::create_all(self.id, self.username.clone(), self.public_key.clone())
    }
}