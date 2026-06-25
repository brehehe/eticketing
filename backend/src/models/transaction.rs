use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub invoice: String,
    pub cashier_id: Uuid,
    pub status: String,
    pub subtotal: i64,
    pub discount: i64,
    pub total: i64,
    pub change: i64,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TransactionItem {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub qty: i32,
    pub unit_price: i64,
    pub discount: i64,
    pub subtotal: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Payment {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub method: String,
    pub provider: Option<String>,
    pub amount: i64,
    pub fee: i64,
    pub reference: Option<String>,
    pub status: String,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub items: Vec<CreateItemRequest>,
    pub payments: Vec<CreatePaymentRequest>,
    pub discount: Option<i64>,
    pub voucher_code: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateItemRequest {
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub qty: i32,
    pub discount: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePaymentRequest {
    pub method: String,
    pub amount: i64,
    pub reference: Option<String>,
}
