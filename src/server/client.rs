use crate::server::redis_hash::{RedisHash, RedisHashContents};

use std::collections::HashMap;
use actix::prelude::*;
use serde::Serialize;

use super::redis_hash::RedisHashContentsUpdate;

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct JsonMessage {
    pub string: String
}

impl JsonMessage {
    pub fn from(serialisable: impl Serialize) -> JsonMessage {
        JsonMessage {
            string: serde_json::to_string(
                &serialisable
            ).unwrap()
        }
    }
}

pub struct Client {
    hash_caches: HashMap<String, RedisHashContents>,
    session: Recipient<JsonMessage>
}

impl Client {
	pub fn new(
		session: Recipient<JsonMessage>
	) -> Client {
		Client {
			hash_caches: HashMap::new(),
			session
		}
	}

    pub fn handle_drop(&mut self, hashname: &String) {
        self.hash_caches.remove(hashname);
    }

	pub fn update_hash(&mut self, hash: &RedisHash) -> bool {
        if let Some(previous_content) = self.hash_caches.insert(
            hash.name.clone(),
            hash.contents.clone()
        ) {
            match RedisHashContentsUpdate::from(
                &hash.contents,
                &previous_content
            ) {
                None => {
                    return false;
                }
                Some(update) => {
                    self.session.do_send(
                        JsonMessage::from(update)
                    );
                }
            }
        } else {
            self.session.do_send(
                JsonMessage::from(hash)
            );
        }
        return true;
	}
}