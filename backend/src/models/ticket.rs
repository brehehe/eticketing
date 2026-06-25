use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Ticket {
    pub id: Uuid,
    pub code: String,
    pub qr_data: String,
    pub transaction_id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub visitor_name: Option<String>,
    pub queue_number: i32,
    pub status: String,
    pub valid_until: Option<DateTime<Utc>>,
    pub used_at: Option<DateTime<Utc>>,
    pub used_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ScanTicketRequest {
    pub code: String,
    pub scan_method: String,
}

#[derive(Debug, Serialize)]
pub struct ScanResult {
    pub result: String, // valid | already_used | expired | invalid
    pub ticket: Option<Ticket>,
    pub message: String,
}
