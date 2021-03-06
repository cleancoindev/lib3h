use errors::*;
use libsodacon::net::endpoint::Endpoint;
use rmp_serde;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DiscoveryReq {
    pub discover: HashMap<Vec<u8>, Vec<Endpoint>>,
}

impl DiscoveryReq {
    pub fn new(discover: HashMap<Vec<u8>, Vec<Endpoint>>) -> Self {
        DiscoveryReq { discover }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DiscoveryRes {
    pub discover: HashMap<Vec<u8>, Vec<Endpoint>>,
}

impl DiscoveryRes {
    pub fn new(discover: HashMap<Vec<u8>, Vec<Endpoint>>) -> Self {
        DiscoveryRes { discover }
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
    DiscoveryReq(Box<DiscoveryReq>),
    DiscoveryRes(Box<DiscoveryRes>),
    UserMessage(Box<UserMessage>),
}

pub fn compile(message: &Message) -> Result<Vec<u8>> {
    Ok(rmp_serde::to_vec(message)?)
}

pub fn parse(message: &[u8]) -> Result<Message> {
    Ok(rmp_serde::from_slice(message)?)
}
