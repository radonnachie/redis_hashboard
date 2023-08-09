use crate::server::redis_hash::{RedisHash, RedisHashContents};

use std::collections::HashMap;
use actix::prelude::*;
use serde::Serialize;

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct JsonMessage {
    pub string: String
}

impl JsonMessage {
    pub fn new(serialisable: &impl Serialize) -> JsonMessage {
        JsonMessage {
            string: serde_json::to_string(
                serialisable
            ).unwrap()
        }
    }

    pub fn from(serialisable: impl Serialize) -> JsonMessage {
        JsonMessage {
            string: serde_json::to_string(
                &serialisable
            ).unwrap()
        }
    }
}

pub struct Client {
    pub hash_caches: HashMap<String, RedisHashContents>,
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

	pub fn update_hash(&self, hash: &RedisHash) {
		self.session.do_send(
			JsonMessage::from(&hash)
		);
	}
}