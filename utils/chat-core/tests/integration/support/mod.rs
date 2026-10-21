pub mod client;
pub mod http;
pub mod people;
pub mod transport;

use std::process::Command;
use std::time::Duration;

use serde_json::Value;

use http::{Reply, Request};

pub use client::Client;
pub use people::Person;

const PLAIN_TIMEOUT: Duration = Duration::from_secs(10);

pub struct Stack {
    pub origin: String,
    psql: Vec<String>,
    admin_token: String,
}

impl Stack {
    pub fn from_env() -> Self {
        let need = |name: &str| {
            std::env::var(name)
                .unwrap_or_else(|_| panic!("{name} is not set, see tests/integration/README.md"))
        };
        let token_file = need("CHAT_IT_ADMIN_TOKEN_FILE");
        assert!(
            std::env::var(chat_core::channel::DEV_OCK_ENV).is_ok(),
            "{} is not set",
            chat_core::channel::DEV_OCK_ENV
        );
        Self {
            origin: need("CHAT_IT_ORIGIN"),
            psql: need("CHAT_IT_PSQL")
                .split_whitespace()
                .map(str::to_owned)
                .collect(),
            admin_token: std::fs::read_to_string(token_file)
                .expect("the admin token file is readable")
                .trim()
                .to_owned(),
        }
    }

    pub fn sql(&self, statement: &str) -> String {
        let output = Command::new(&self.psql[0])
            .args(&self.psql[1..])
            .args(["-v", "ON_ERROR_STOP=1", "-qAt", "-c", statement])
            .output()
            .expect("psql runs");
        assert!(
            output.status.success(),
            "{statement}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    }

    pub async fn admin(&self, method: &str, path: &str, body: Option<Value>) -> Reply {
        let mut headers = vec![
            ("x-chat-admin-token", self.admin_token.clone()),
            ("x-chat-admin-user", "integration".to_owned()),
        ];
        if body.is_some() {
            headers.push(("content-type", "application/json".to_owned()));
        }
        self.plain(method, path, headers, body).await
    }

    pub async fn as_session(
        &self,
        session_id: uuid::Uuid,
        method: &str,
        path: &str,
        body: Option<Value>,
    ) -> Reply {
        let headers = vec![
            ("x-session-id", session_id.to_string()),
            ("content-type", "application/json".to_owned()),
        ];
        self.plain(method, path, headers, body).await
    }

    async fn plain(
        &self,
        method: &str,
        path: &str,
        headers: Vec<(&str, String)>,
        body: Option<Value>,
    ) -> Reply {
        let request = Request {
            method,
            path,
            headers,
            body: body
                .map(|body| body.to_string().into_bytes())
                .unwrap_or_default(),
            timeout: PLAIN_TIMEOUT,
        };
        http::send(&self.origin, request)
            .await
            .unwrap_or_else(|error| panic!("{method} {path}: {error}"))
    }
}

pub async fn eventually<F, Fut>(what: &str, mut check: F)
where
    F: FnMut() -> Fut,
    Fut: Future<Output = bool>,
{
    for _ in 0..40 {
        if check().await {
            return;
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    panic!("{what} did not happen within 10 s");
}
