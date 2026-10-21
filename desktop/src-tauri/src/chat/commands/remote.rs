use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use chat_core::api::CallOptions;
use chat_core::channel::InnerCall;
use chat_core::{ChatError, ChatResult, ConvId};

use crate::chat::state::ChatState;

pub const GET: &str = "GET";
pub const POST: &str = "POST";
pub const PUT: &str = "PUT";
pub const DELETE: &str = "DELETE";

pub async fn json<T, B>(
    state: &ChatState,
    method: &str,
    path: &str,
    query: &str,
    body: Option<&B>,
) -> ChatResult<T>
where
    T: DeserializeOwned,
    B: Serialize,
{
    let answer = exchange(state, method, path, query, body).await?;
    serde_json::from_slice(&answer).map_err(|_| ChatError::Protocol)
}

pub async fn done<B: Serialize>(
    state: &ChatState,
    method: &str,
    path: &str,
    query: &str,
    body: Option<&B>,
) -> ChatResult<()> {
    exchange(state, method, path, query, body).await.map(|_| ())
}

async fn exchange<B: Serialize>(
    state: &ChatState,
    method: &str,
    path: &str,
    query: &str,
    body: Option<&B>,
) -> ChatResult<Vec<u8>> {
    let body = match body {
        Some(body) => serde_json::to_vec(body).map_err(|_| ChatError::Invalid)?,
        None => Vec::new(),
    };
    let call = InnerCall {
        method: method.to_string(),
        path: path.to_string(),
        query: query.to_string(),
        body,
    };
    let answer = state.api.call(call, CallOptions::default()).await?;
    if answer.status >= 400 {
        let error = serde_json::from_slice(&answer.body).unwrap_or(Value::Null);
        return Err(ChatError::from_server(&error));
    }
    Ok(answer.body)
}

pub fn segment(raw: &str) -> String {
    urlencoding::encode(raw).into_owned()
}

pub fn conversation(conversation_id: ConvId) -> String {
    format!("/chat/conversations/{conversation_id}")
}

pub const NO_BODY: Option<&()> = None;
