//! `RedisHashBroker` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `RedisHashBroker`.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{mpsc::{self, Sender, TryRecvError}, Mutex, Arc},
    thread
};

use actix::prelude::*;
use serde::Serialize;

use redis::Commands;

pub type HashNameSet = HashSet<String>;

pub enum SessionMessages {
    Disconnect,
    Connect(Recipient<JsonMessage>),
    Request(HashNameSet),
    Drop(HashNameSet)
}

pub struct SessionMessage {
    pub id: usize,
    pub message: SessionMessages
}

type HashUpdateMap = HashMap<String, HashMap<String, String>>;

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

struct Client {
    running_hashrequests: HashSet<String>,
    session: Recipient<JsonMessage>
}

pub struct RedisHashBroker {
    next_client_id: Arc<Mutex<usize>>,
    redis_thread: thread::JoinHandle<()>,
    tx: Sender<SessionMessage>
}

impl RedisHashBroker {
    pub fn new() -> RedisHashBroker {
        
        let (tx, rx) = mpsc::channel();

        RedisHashBroker {
            next_client_id: Arc::new(Mutex::new(0)),
            redis_thread: thread::spawn(move || {
                let mut clients: HashMap<usize, Client> = HashMap::new();
                let mut hashrequest_clients: HashMap<String, HashSet<usize>> = HashMap::new();
                let mut hashrequest_queue: VecDeque<String> = VecDeque::new();

                let redis_client = redis::Client::open("redis://localhost:6379").unwrap();
                let mut redis_connection = redis_client.get_connection().unwrap();

                loop {
                    match rx.try_recv() {
                        Ok(SessionMessage {
                            id,
                            message: SessionMessages::Disconnect
                        }) => {
                            if clients.remove(&id).is_some() {
                                for (_hash, clients) in &mut hashrequest_clients {
                                    clients.remove(&id);
                                }
                            }
                        },

                        Ok(SessionMessage {
                            id,
                            message: SessionMessages::Connect(addr)
                        }) => {
                            clients.insert(
                                id,
                                Client {
                                    running_hashrequests: HashSet::new(),
                                    session: addr
                                }
                            );
                        },

                        Ok(SessionMessage {
                            id,
                            message: SessionMessages::Request(hashes)
                        }) => {
                            let mut hash_updates = HashUpdateMap::new();
                            let client = clients.get_mut(&id).unwrap();
                            
                            for hash in hashes {
                                // file client's hash-requests
                                hashrequest_clients
                                    .entry(hash.clone())
                                    .or_insert_with(HashSet::new)
                                    .insert(id);

                                // if a novel request, push cached hash-contents
                                // to the hash-update to be messaged
                                if !client.running_hashrequests.contains(&hash) {
                                    if let Ok(hash_contents) = redis_connection.hgetall(&hash) {
                                        hash_updates.insert(
                                            hash.to_owned(),
                                            hash_contents  
                                        );
                                    }
                                    
                                    client.running_hashrequests.insert(hash.clone());
                                }

                                if !hashrequest_queue.contains(&hash) {
                                    hashrequest_queue.push_back(hash);
                                }
                            }
                            if hash_updates.len() > 0 {
                                client.session.do_send(
                                    JsonMessage::new(&hash_updates)
                                );
                            }
                        },

                        Ok(SessionMessage {
                            id,
                            message: SessionMessages::Drop(hashes)
                        }) => {
                            let client = clients.get_mut(&id).unwrap();
                            
                            for hash in hashes {
                                // remove from running list
                                client.running_hashrequests.remove(&hash);

                                // file client's hash-requests
                                let hash_clients = hashrequest_clients.get_mut(&hash).unwrap();
                                hash_clients.remove(&id);
                            }
                        },

                        Err(TryRecvError::Empty) => (),
                        Err(TryRecvError::Disconnected) => {
                            break;
                        }
                    }
                
                    if let Some(hash) = hashrequest_queue.pop_front() {
                        let hash_clients = hashrequest_clients.get_mut(&hash).unwrap();
                        if hash_clients.len() == 0 {
                            hashrequest_clients.remove(&hash);
                            break;
                        }

                        let hashcontent: HashMap<String, String> = redis_connection.hgetall(&hash).unwrap();
                        let json = JsonMessage::from(
                            HashUpdateMap::from([
                                (hash.clone(), hashcontent)
                            ])
                        );
                        
                        for client in hash_clients.drain() {
                            match clients.get(&client) {
                                Some(client) => {
                                    client.session.do_send(json.clone());
                                },
                                None => {
                                    // should probably error
                                }
                            }
                        }
                    }
                }
            }),
            tx: tx
        }
    }

    pub fn clone_tx(&self) -> Sender<SessionMessage> {
        self.tx.clone()
    }

    pub fn take_next_client_id(&self) -> usize {
        let mut next_client_id = self.next_client_id.lock().unwrap();
        let client_id = *next_client_id;
        *next_client_id += 1;
        client_id
    }
}

impl Default for RedisHashBroker {
    fn default() -> Self {
        Self::new()
    }
}
