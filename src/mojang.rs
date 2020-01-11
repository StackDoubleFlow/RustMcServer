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
    pub clientId: u32,
    requestType: RequestType,
    future: ReqwestFuture,
    pub result: Option<R>,
}

impl<R> Pending<R> {
    fn new(clientId: u32, future: ReqwestFuture, requestType: RequestType) -> Pending<R> {
        Pending {
            clientId,
            future,
            requestType,
            result: None,
        }
    }

    async fn run(&mut self) {
        //self.future.await;
    }
}

pub struct Mojang {
    reqClient: reqwest::Client,
    pub hasJoinedPending: Vec<Pending<MojangHasJoinedResponse>>,
}

impl Mojang {
    pub fn new() -> Self {
        Mojang {
            reqClient: reqwest::Client::new(),
            hasJoinedPending: Vec::new(),
        }
    }

    pub fn send_has_joined(&mut self, username: &String, clientId: u32) {
        let url = format!(
            "https://sessionserver.mojang.com/session/minecraft/hasJoined?username={}&serverId={}",
            username,
            mc_hex_digest(&username)
        );

        let future = self.reqClient.get(&url).send();
        let pending = Pending::new(clientId, future.boxed(), RequestType::HasJoined);
        self.hasJoinedPending.push(pending);
    }

    pub fn clean(&mut self) {
        self.hasJoinedPending.retain(|p| p.result.is_some());
    }
}
