use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Default, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub id: Option<Uuid>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub path: Option<String>,
    pub size: Option<i32>,
    pub organization_id: Option<Uuid>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}
