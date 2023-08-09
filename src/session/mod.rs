pub mod client_action;

use std::{time::{Duration, Instant}, sync::mpsc::Sender};

use actix::prelude::*;
use actix_web_actors::ws;

use crate::{server::{self, SessionMessages, SessionMessage}, session::client_action::ClientAction};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WsChatSession {
    /// unique session id
    pub id: usize,

    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    pub hb: Instant,

    /// Sender to the RedisHashBroker
    pub tx: Sender<SessionMessage>,
}

impl WsChatSession {
    /// helper method that sends ping to client every 5 seconds (HEARTBEAT_INTERVAL).
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // // notify chat server
                // act.tx.client_deregistration.send(server::Disconnect { id: act.id });

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    /// We register ws session with RedisHashBroker
    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);

        // Send self registration details to RedisHashBroker
        let _ = self.tx.send(
            SessionMessage {
                id: self.id,
                message: SessionMessages::Connect(
                    ctx.address().recipient()
                ),
            }
        );
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify server
        let _ = self.tx.send(
            SessionMessage {
                id: self.id,
                message: SessionMessages::Disconnect,
            }
        );
        Running::Stop
    }
}

/// Handle JsonMessage from RedisHashBroker
impl Handler<server::JsonMessage> for WsChatSession {
    type Result = ();

    fn handle(&mut self, json: server::JsonMessage, ctx: &mut Self::Context) {
        ctx.text(json.string);
    }
}

/// WebSocket message handler
/// Handles messages from the client
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        log::debug!("WEBSOCKET MESSAGE: {msg:?}");
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                match serde_json::from_str::<ClientAction>(&text) {
                    Ok(action) => {
                        let _ = self.tx.send(
                            SessionMessage {
                                id: self.id,
                                message: SessionMessages::Action(action)
                            }
                        );
                    },
                    Err(err) => {
                        ctx.text(format!("!!! {:}", err));
                    }
                }
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
