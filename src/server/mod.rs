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
        redis_hash::RedisHash,
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
                            for hash in hash_names {
                                // file client's hash-requests
                                hashrequest_clients
                                    .entry(hash.clone())
                                    .or_insert_with(HashSet::new)
                                    .insert(id);

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
                                client.handle_drop(&hash);

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
                        let mut hash_clients = hashrequest_clients.remove(&hash).unwrap();
                        if hash_clients.len() > 0 {
                            let redishash = RedisHash {
                                name: hash.clone(),
                                contents: redis_connection.hgetall(&hash).unwrap()
                            };
                            
                            for clientid in hash_clients.drain() {
                                match clients.get_mut(&clientid) {
                                    Some(client) => {
                                        if !client.update_hash(&redishash) {
                                            // no update, re-constitute request
                                            hashrequest_queue.push_back(hash.clone());

                                            hashrequest_clients
                                                .entry(hash.clone())
                                                .or_insert_with(HashSet::new)
                                                .insert(clientid);
                                        }
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
