use chat_wire::proto::Envelope;

use crate::contracts::{
    ChatError, ChatResult, ConversationMeta, Embed, Forwarded, MessageClass, RichText, Seq, UserId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Post {
    pub text: RichText,
    pub reply_to: Option<Seq>,
    pub mentions: Vec<UserId>,
    pub embeds: Vec<Embed>,
    pub forwarded_from: Option<Forwarded>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    Post(Post),
    Edit {
        target: Seq,
        text: RichText,
    },
    Delete {
        target: Seq,
    },
    Reaction {
        target: Seq,
        emoji: String,
        on: bool,
    },
    Meta(ConversationMeta),
    Pin {
        target: Seq,
        on: bool,
    },
    Suggest(Embed),
    Unknown,
}

impl Content {
    pub fn class(&self) -> MessageClass {
        match self {
            Content::Post(_) | Content::Suggest(_) | Content::Unknown => MessageClass::Post,
            Content::Edit { .. } | Content::Delete { .. } => MessageClass::Edit,
            Content::Reaction { .. } => MessageClass::React,
            Content::Meta(_) => MessageClass::Meta,
            Content::Pin { .. } => MessageClass::Pin,
        }
    }

    pub fn target(&self) -> Option<Seq> {
        match self {
            Content::Edit { target, .. }
            | Content::Delete { target }
            | Content::Reaction { target, .. }
            | Content::Pin { target, .. } => Some(*target),
            Content::Post(_) | Content::Meta(_) | Content::Suggest(_) | Content::Unknown => None,
        }
    }
}

pub fn encode(_content: &Content) -> ChatResult<Envelope> {
    Err(ChatError::Unimplemented)
}

pub fn decode(_envelope: &Envelope) -> ChatResult<Content> {
    Err(ChatError::Unimplemented)
}
