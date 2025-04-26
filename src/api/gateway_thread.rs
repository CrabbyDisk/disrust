/*
op codes cheat sheet:
0: gateway event
1: heartbeat sent
2: ready event (A load of info like guilds, user, settings, etc)
10: discord sent you heartbeat interval, hello
11: discord received your heartbeat

The gateway events are identified by string names
VC has its own op codes

btw people's email address is public through the api I think, weird
*/

use futures::{SinkExt, StreamExt};
use serde_json::{self, Value};
use std::time::Instant;
use tokio::spawn;
use tokio::sync::mpsc;
use tokio::{net::TcpStream, sync::mpsc::UnboundedSender};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use url::Url;

use crate::api::data::*;

pub async fn start_thread(tx: UnboundedSender<GatewayResponse>, token: &str) {
    let gateway_url = Url::parse("wss://gateway.discord.gg/?v=9&encoding=json").unwrap();
    let (mut socket, response) = connect_async(gateway_url).await.expect("Can't connect");

    //Not sure if it's correct terminology
    let handshake = read_json_event(&mut socket).await.unwrap();
    let hb_interval = handshake["d"]["heartbeat_interval"].as_i64().unwrap();
    println!("Received Hbeat: {}", hb_interval);

    identify(&mut socket, token).await;

    //Can get a lot of data from it in order to
    //not update much in network_thread
    let ready = read_json_event(&mut socket)
        .await
        .expect("Couldn't get ready event");
    ready_event(&tx, ready);

    spawn(async move {
        let mut timer = Instant::now();
        loop {
            let event = read_json_event(&mut socket).await;
            // dbg!(&event);
            if event.is_err() {
                println!("Gateway disconnected");
                continue;
            }

            let event = event.unwrap();

            let op_code = event["op"].as_i64().unwrap();
            // dbg!(op_code);
            if op_code == 1 {
                heartbeat(&mut socket).await;
            }

            //Should put all the events in a list or smthn
            if op_code == 0 {
                let event_name = event["t"].as_str().unwrap();
                match event_name {
                    "MESSAGE_CREATE" => {
                        message_created(&tx, &event);
                    }
                    "MESSAGE_REACTION_ADD" => (),
                    "MESSAGE_REACTION_REMOVE" => (),
                    "TYPING_START" => (),
                    "CHANNEL_CREATE" => (),
                    "GUILD_CREATE" => (),
                    "GUILD_DELETE" => (),
                    _ => (),
                }
            }

            //Heartbeat here
            //A thread would have to borrow the socket and it was a pain
            let elapsed = timer.elapsed().as_millis() as i64;
            if hb_interval <= elapsed {
                heartbeat(&mut socket).await;
                timer = Instant::now();
            }
        }
    });
}

//Each event has an attached sequence number
//Heartbeats need to include latest sequence number
//^^^ Didn't use it in python test and had no problems. Abandoned for now
async fn heartbeat(socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) {
    let reply = Message::Text(
        r#"{
        "op": 1,
        "d": "null"
    }"#
        .into(),
    );

    socket.send(reply).await.expect("Hbeat failed");
}

async fn identify(socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>, token: &str) {
    //ugly as fuck
    let reply = format!(
        "{{
        \"op\": 2,
        \"d\": {{
            \"token\": \"{}\",
            \"properties\": {{
                \"$os\": \"linux\",
                \"$browser\": \"chrome\",
                \"$device\": \"pc\"
            }}
        }}
    }}",
        token
    );

    let reply = Message::Text(reply);

    socket.send(reply).await.expect("Identification failed");
}

//Makes a Msg object and sends it back to ui thread
fn message_created(tx: &mpsc::UnboundedSender<GatewayResponse>, event: &Value) {
    let msg = Msg::from(&event["d"]);
    let gate_response = GatewayResponse::msg_create(msg);
    tx.send(gate_response).unwrap();
}

fn ready_event(tx: &mpsc::UnboundedSender<GatewayResponse>, event: Value) {
    let guilds = Guild::from_list(&event["d"]);
    let gate_response = GatewayResponse::ready(guilds);
    tx.send(gate_response).unwrap();
}

// use result instead
// Some weird shit with gateway disconnect idk
async fn read_json_event(
    socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) -> Result<serde_json::Value, serde_json::Error> {
    let msg = socket.next().await;
    let msg = msg.expect("Error reading msg").unwrap();
    let text_msg = msg.to_text().expect("No text, I think");

    serde_json::from_str(text_msg)
}
