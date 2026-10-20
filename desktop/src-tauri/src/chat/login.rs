use chat_core::{
    ChatError, ChatResult, LinkClaimed, LinkCreated, LinkPoll, LoginBegin, LoginPoll, LoginTarget,
};

use super::state::ChatState;

pub async fn begin(_state: &ChatState, _target: LoginTarget) -> ChatResult<LoginBegin> {
    Err(ChatError::Unimplemented)
}

pub async fn poll(_state: &ChatState, _login_request_id: &str) -> ChatResult<LoginPoll> {
    Err(ChatError::Unimplemented)
}

pub async fn link_create_pull(_state: &ChatState) -> ChatResult<LinkCreated> {
    Err(ChatError::Unimplemented)
}

pub async fn link_claim_push(_state: &ChatState, _claim_token: &str) -> ChatResult<LinkClaimed> {
    Err(ChatError::Unimplemented)
}

pub async fn link_poll(_state: &ChatState, _link_request_id: &str) -> ChatResult<LinkPoll> {
    Err(ChatError::Unimplemented)
}
