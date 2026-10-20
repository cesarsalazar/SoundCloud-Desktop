use crate::contracts::{ChatError, ChatResult, Embed};

pub fn validate(_embed: &Embed) -> ChatResult<Embed> {
    Err(ChatError::Unimplemented)
}

pub fn validate_all(embeds: &[Embed]) -> Vec<Embed> {
    embeds
        .iter()
        .filter_map(|embed| validate(embed).ok())
        .collect()
}
