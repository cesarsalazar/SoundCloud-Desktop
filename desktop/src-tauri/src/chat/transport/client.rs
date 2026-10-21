use std::sync::OnceLock;
use std::time::Duration;

use chat_core::{TransportError, TransportResponse};
use futures_util::{Stream, StreamExt};
use wreq::header::{CONTENT_TYPE, HeaderMap, RETRY_AFTER};
use wreq::redirect::Policy;

use super::hops::Hop;
use crate::network::fail::{self, FailKind};
use crate::network::pace::Pace;
use crate::network::pro::{self, Asked};
use crate::network::{fetch, system_proxy};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const IDLE_PER_HOST: usize = 2;
const OCTET_STREAM: &str = "application/octet-stream";
pub const ANSWER_MAX: usize = 2 * 1024 * 1024;

pub async fn send(hop: &Hop, envelope: &[u8]) -> Result<TransportResponse, TransportError> {
    match hop {
        Hop::Http(url) => post(url, envelope).await,
        Hop::Pro { node, url } => through_pro(node, url, envelope).await,
    }
}

fn client() -> Option<&'static wreq::Client> {
    static CLIENT: OnceLock<Option<wreq::Client>> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            let (_, emulation) = sc_fingerprint::emulation(None);
            let builder = sc_fingerprint::from_emulation(fetch::pinging(emulation, Pace::Patient));
            system_proxy::follow(builder)
                .redirect(Policy::none())
                .connect_timeout(CONNECT_TIMEOUT)
                .pool_max_idle_per_host(IDLE_PER_HOST)
                .build()
                .ok()
        })
        .as_ref()
}

async fn post(url: &str, envelope: &[u8]) -> Result<TransportResponse, TransportError> {
    let client = client().ok_or_else(|| TransportError::Other("no http client".into()))?;
    let response = client
        .post(url)
        .header(CONTENT_TYPE, OCTET_STREAM)
        .body(envelope.to_vec())
        .send()
        .await
        .map_err(|error| failure(&error))?;
    let status = response.status().as_u16();
    let retry_after_ms = retry_after(response.headers());
    if lost_on_the_way(status, retry_after_ms) {
        return Err(TransportError::Unreachable);
    }
    if response
        .content_length()
        .is_some_and(|length| length > ANSWER_MAX as u64)
    {
        return Err(TransportError::Unreachable);
    }
    let body = gather(response.bytes_stream(), |error| failure(&error)).await?;
    Ok(TransportResponse {
        status,
        body,
        retry_after_ms,
    })
}

async fn through_pro(
    node: &str,
    url: &str,
    envelope: &[u8],
) -> Result<TransportResponse, TransportError> {
    let asked = Asked {
        method: "POST".to_string(),
        url: url.to_string(),
        headers: vec![(CONTENT_TYPE.as_str().to_string(), OCTET_STREAM.to_string())],
        body: Some(envelope.to_vec()),
    };
    let answer = pro::fetch(node, asked)
        .await
        .map_err(|_| TransportError::Unreachable)?;
    let retry_after_ms = answer
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(RETRY_AFTER.as_str()))
        .and_then(|(_, value)| seconds_to_ms(value));
    if lost_on_the_way(answer.status, retry_after_ms) {
        return Err(TransportError::Unreachable);
    }
    let body = gather(answer.body, |_| TransportError::Unreachable).await?;
    Ok(TransportResponse {
        status: answer.status,
        body,
        retry_after_ms,
    })
}

pub async fn gather<S, P, E>(
    pieces: S,
    lost: impl Fn(E) -> TransportError,
) -> Result<Vec<u8>, TransportError>
where
    S: Stream<Item = Result<P, E>>,
    P: AsRef<[u8]>,
{
    let mut pieces = std::pin::pin!(pieces);
    let mut body = Vec::new();
    while let Some(piece) = pieces.next().await {
        let piece = piece.map_err(&lost)?;
        if body.len() + piece.as_ref().len() > ANSWER_MAX {
            return Err(TransportError::Unreachable);
        }
        body.extend_from_slice(piece.as_ref());
    }
    Ok(body)
}

pub fn lost_on_the_way(status: u16, retry_after_ms: Option<u64>) -> bool {
    match status {
        421 => true,
        502..=504 => retry_after_ms.is_none(),
        _ => false,
    }
}

fn retry_after(headers: &HeaderMap) -> Option<u64> {
    headers
        .get(RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(seconds_to_ms)
}

pub fn seconds_to_ms(value: &str) -> Option<u64> {
    value
        .trim()
        .parse::<u64>()
        .ok()
        .map(|s| s.saturating_mul(1000))
}

fn failure(error: &wreq::Error) -> TransportError {
    match fail::of_wreq(error) {
        FailKind::Timeout => TransportError::Timeout,
        FailKind::Status | FailKind::Body | FailKind::Other => {
            TransportError::Other(error.to_string())
        }
        _ => TransportError::Unreachable,
    }
}
