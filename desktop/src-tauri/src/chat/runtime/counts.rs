use chat_core::store::Store;
use chat_core::{
    ChatResult, ConversationKind, ConversationSummary, ListFilter, ListQuery, Millis, NotifyLevel,
};

const PAGE: u32 = 200;
const PAGES_MAX: usize = 10;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    pub unread: u32,
    pub mentions: u32,
    pub requests: u32,
}

pub fn count(store: &Store, now: Millis) -> ChatResult<Counts> {
    let mut counts = Counts::default();
    for summary in all(store, ListFilter::All)? {
        counts.mentions = counts.mentions.saturating_add(summary.mentions);
        if !quiet(&summary, now) {
            counts.unread = counts.unread.saturating_add(summary.unread);
        }
    }
    let requests = all(store, ListFilter::Requests)?.len();
    counts.requests = u32::try_from(requests).unwrap_or(u32::MAX);
    Ok(counts)
}

pub fn quiet(summary: &ConversationSummary, now: Millis) -> bool {
    summary.archived
        || !summary.accepted
        || summary.kind == ConversationKind::Room
        || summary.notify == NotifyLevel::None
        || summary.muted_until.is_some_and(|until| until > now)
}

fn all(store: &Store, filter: ListFilter) -> ChatResult<Vec<ConversationSummary>> {
    let mut items = Vec::new();
    let mut cursor = None;
    for _ in 0..PAGES_MAX {
        let page = store.conversations(&ListQuery {
            filter,
            cursor,
            limit: PAGE,
        })?;
        items.extend(page.items);
        match page.next_cursor {
            Some(next) => cursor = Some(next),
            None => break,
        }
    }
    Ok(items)
}
