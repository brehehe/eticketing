use chrono::{DateTime, Utc, Duration};

/// Determine valid_until based on expiry_hours setting (0 = no expiry)
pub fn compute_expiry(expiry_hours: i64) -> Option<DateTime<Utc>> {
    if expiry_hours <= 0 { return None; }
    Some(Utc::now() + Duration::hours(expiry_hours))
}

/// Check if a ticket is still valid
pub fn is_valid(status: &str, valid_until: Option<DateTime<Utc>>) -> bool {
    if status != "active" { return false; }
    if let Some(exp) = valid_until {
        return Utc::now() < exp;
    }
    true
}
