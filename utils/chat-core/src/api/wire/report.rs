use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileReport {
    pub conversation_id: Uuid,
    pub seq_from: i64,
    pub seq_to: i64,
    pub reason: String,
    pub comment: Option<String>,
}
