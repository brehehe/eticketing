/// Generate sequential invoice number with prefix
pub fn generate_invoice(prefix: &str, count: i64) -> String {
    format!("{}-{:07}", prefix, count + 1)
}

/// Generate unique ticket code
pub fn generate_ticket_code(invoice: &str, index: i32) -> String {
    format!("{}-{:03}", invoice, index + 1)
}

/// Build QR payload for a ticket
pub fn build_qr_data(transaction_id: &str, code: &str) -> String {
    format!("TIKETKU:{}:{}", transaction_id, code)
}
