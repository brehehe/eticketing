use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

pub fn deserialize_double_option<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Option::deserialize(deserializer).map(Some)
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub sku: String,
    pub category_id: Option<Uuid>,
    pub price: i64,
    pub investor_price: i64,
    pub investor_id: Option<Uuid>,
    pub quota: Option<i32>,
    pub quota_used: i32,
    pub schedule: Option<String>,
    pub status: String,
    pub has_variants: bool,
    pub ticket_required: bool,
    pub print_enabled: bool,
    pub scan_enabled: bool,
    pub barcode: Option<String>,
    pub image: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductVariant {
    pub id: Uuid,
    pub product_id: Uuid,
    pub name: String,
    pub sku: String,
    pub price: i64,
    pub investor_price: i64,
    pub quota: Option<i32>,
    pub quota_used: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub sku: String,
    pub category_id: Option<Uuid>,
    pub price: i64,
    pub investor_price: Option<i64>,
    pub investor_id: Option<Uuid>,
    pub quota: Option<i32>,
    pub status: Option<String>,
    pub has_variants: Option<bool>,
    pub ticket_required: Option<bool>,
    pub print_enabled: Option<bool>,
    pub scan_enabled: Option<bool>,
    pub variants: Option<Vec<CreateVariantRequest>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVariantRequest {
    pub name: String,
    pub sku: String,
    pub price: i64,
    pub investor_price: Option<i64>,
    pub quota: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub price: Option<i64>,
    pub investor_price: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_double_option")]
    pub investor_id: Option<Option<Uuid>>,
    #[serde(default, deserialize_with = "deserialize_double_option")]
    pub quota: Option<Option<i32>>,
    pub status: Option<String>,
    pub ticket_required: Option<bool>,
    pub print_enabled: Option<bool>,
    pub scan_enabled: Option<bool>,
}
