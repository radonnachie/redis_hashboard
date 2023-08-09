mod redis_hash;
pub mod client;

use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{mpsc::{self, Sender, TryRecvError}, Mutex, Arc},
    thread
};

use actix::prelude::*;

use redis::Commands;

use crate::{
    server::{
        redis_hash::{RedisHash, RedisHashContents},
        client::{Client, JsonMessage}
    },
    session::client_action::{ClientAction, ClientActions}
};

pub enum SessionMessages {
    Disconnect,
    Connect(Recipient<JsonMessage>),
    Action(ClientAction)
}

pub struct SessionMessage {
    pub id: usize,
    pub message: SessionMessages
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

                let redis_client = redis::Client::open("redis://redishost:6379").unwrap();
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
                                Client::new(addr)
                            );
                        },

                        Ok(SessionMessage {
                            id,
                            message: SessionMessages::Action(
                                ClientAction {
                                    action : ClientActions::Request,
                                    hash_names
                                }
                            )
                        }) => {
                            let client = clients.get_mut(&id).unwrap();
                            
                            for hash in hash_names {
                                // file client's hash-requests
                                hashrequest_clients
                                    .entry(hash.clone())
                                    .or_insert_with(HashSet::new)
                                    .insert(id);

                                // if a novel request, push cached hash-contents
                                // to the hash-update to be messaged
                                if !client.hash_caches.contains_key(&hash) {
                                    client.hash_caches.insert(
                                        hash.clone(),
                                        RedisHashContents::new()
                                    );
                                }

                                if !hashrequest_queue.contains(&hash) {
                                    hashrequest_queue.push_back(hash);
                                }
                            }
                        },

                        Ok(SessionMessage {
                            id,
                            message: SessionMessages::Action(
                                ClientAction {
                                    action : ClientActions::Drop,
                                    hash_names
                                }
                            )
                        }) => {
                            let client = clients.get_mut(&id).unwrap();
                            
                            for hash in hash_names {
                                // remove from running list
                                client.hash_caches.remove(&hash);

                                // remove from hash's clients
                                if let Some(hash_clients) = hashrequest_clients.get_mut(&hash) {
                                    hash_clients.remove(&id);
                                }
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
                        }
                        else {
                            let redishash = RedisHash {
                                name: hash.clone(),
                                contents: redis_connection.hgetall(&hash).unwrap()
                            };
                            
                            for client in hash_clients.drain() {
                                match clients.get(&client) {
                                    Some(client) => {
                                        client.update_hash(&redishash);
                                    },
                                    None => {
                                        // should probably error
                                    }
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
