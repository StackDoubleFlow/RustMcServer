use crate::utils::mc_hex_digest;
use futures::{Future, FutureExt};
use reqwest::Response;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::pin::Pin;

type MojangResult<T> = std::result::Result<T, MojangError>;

#[derive(Debug)]
pub enum MojangError {
    ConnectionError,
}

impl fmt::Display for MojangError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "") // TODO: Error Message
    }
}

impl Error for MojangError {}

#[derive(Serialize, Deserialize)]
pub struct MojangHasJoinedResponseProperties {
    name: String,
    value: String,
    signature: String,
}

#[derive(Serialize, Deserialize)]
pub struct MojangHasJoinedResponse {
    id: String,
    name: String,
    properties: Vec<MojangHasJoinedResponseProperties>,
}

enum RequestType {
    HasJoined,
}

type ReqwestFuture = Pin<Box<dyn Future<Output = Result<Response, reqwest::Error>>>>;

pub struct Pending<R> {
    pub client_id: u32,
    request_type: RequestType,
    future: ReqwestFuture,
    pub result: Option<R>,
}

impl<R> Pending<R> {
    fn new(client_id: u32, future: ReqwestFuture, request_type: RequestType) -> Pending<R> {
        Pending {
            client_id,
            future,
            request_type,
            result: None,
        }
    }

    async fn run(&mut self) {
        //self.future.await;
    }
}

pub struct Mojang {
    req_client: reqwest::Client,
    pub has_joined_pending: Vec<Pending<MojangHasJoinedResponse>>,
}

impl Mojang {
    pub fn new() -> Self {
        Mojang {
            req_client: reqwest::Client::new(),
            has_joined_pending: Vec::new(),
        }
    }

    pub fn send_has_joined(&mut self, username: &String, client_id: u32) {
        let url = format!(
            "https://sessionserver.mojang.com/session/minecraft/hasJoined?username={}&serverId={}",
            username,
            mc_hex_digest(&username)
        );

        let future = self.req_client.get(&url).send();
        let pending = Pending::new(client_id, future.boxed(), RequestType::HasJoined);
        self.has_joined_pending.push(pending);
    }

    pub fn clean(&mut self) {
        self.has_joined_pending.retain(|p| p.result.is_some());
    }
}
