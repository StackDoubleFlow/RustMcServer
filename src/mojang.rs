use crate::utils::mc_hex_digest;
use futures::{Future, FutureExt};
use reqwest::Response;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

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

struct Pending {
    clientId: u32,
    requestType: RequestType,
    future: ReqwestFuture,
}

impl Pending {
    fn new(clientId: u32, future: ReqwestFuture, requestType: RequestType) -> Pending {
        Pending {
            clientId,
            future,
            requestType,
        }
    }

    fn poll(&self) {
        self.future.as_mut().poll();
    }
}

pub struct Mojang {
    reqClient: reqwest::Client,
    pendecies: Vec<Pending>,
}

impl Mojang {
    pub fn new() -> Mojang {
        Mojang {
            reqClient: reqwest::Client::new(),
            pendecies: Vec::new(),
        }
    }

    pub fn poll(&self) {}

    pub fn sendHasJoined(&mut self, username: String, clientId: u32) {
        let url = format!(
            "https://sessionserver.mojang.com/session/minecraft/hasJoined?username={}&serverId={}",
            username,
            mc_hex_digest(&username)
        );

        let future = self.reqClient.get(&url).send();
        future.poll();
        let pending = Pending::new(clientId, future, RequestType::HasJoined);
        self.pendecies.push(pending);
    }
}
