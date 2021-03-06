use errors::*;
use libsodacrypt;
use net::endpoint::Endpoint;
use net::http;
use rmp_serde;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_millis() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    let nanos: u64 = since_the_epoch.subsec_nanos().into();
    since_the_epoch.as_secs() * 1000 + nanos / 1_000_000
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InitialHandshakeRes {
    pub session_id: String,
    pub node_id: Vec<u8>,
    pub eph_pub: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PingReq {
    pub sent_time: u64,
    pub node_id: Vec<u8>,
    pub discover: Vec<Endpoint>,
}

impl PingReq {
    pub fn new(node_id: &[u8], discover: Vec<Endpoint>) -> Self {
        PingReq {
            sent_time: get_millis(),
            node_id: node_id.to_vec(),
            discover,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PingRes {
    pub origin_time: u64,
    pub response_time: u64,
    pub node_id: Vec<u8>,
    pub discover: Vec<Endpoint>,
}

impl PingRes {
    pub fn new(origin_time: u64, node_id: &[u8], discover: Vec<Endpoint>) -> Self {
        PingRes {
            origin_time,
            response_time: get_millis(),
            node_id: node_id.to_vec(),
            discover,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UserMessage {
    pub data: Vec<u8>,
}

impl UserMessage {
    pub fn new(data: Vec<u8>) -> Self {
        UserMessage { data }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Message {
    PingReq(Box<PingReq>),
    PingRes(Box<PingRes>),
    UserMessage(Box<UserMessage>),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct MsgWrap(Vec<u8>, Vec<u8>);

pub fn compile(
    session_id: &str,
    sub_messages: &[Message],
    rtype: http::RequestType,
    psk: &[u8],
) -> Result<Vec<u8>> {
    let msg = rmp_serde::to_vec(sub_messages)?;

    let (nonce, msg) = libsodacrypt::sym::enc(&msg, psk)?;
    let msg = rmp_serde::to_vec(&MsgWrap(nonce, msg))?;

    let mut req_out = http::Request::new(rtype);
    req_out.method = "POST".to_string();
    req_out.path = format!("/{}", session_id);
    req_out.code = "200".to_string();
    req_out.status = "OK".to_string();
    req_out.headers.insert(
        "content-type".to_string(),
        "application/octet-stream".to_string(),
    );
    req_out.body = msg;

    let msg = req_out.generate();

    Ok(msg)
}

pub fn parse(message: &[u8], psk: &[u8]) -> Result<Vec<Message>> {
    let message: MsgWrap = rmp_serde::from_slice(message)?;
    let message = libsodacrypt::sym::dec(&message.1, &message.0, psk)?;
    let message: Vec<Message> = rmp_serde::from_slice(&message)?;
    Ok(message)
}
