-- ============================================================
-- TiketKu — Migration 002
-- Adds footer lines to venue settings, logo field,
-- and new notification & payment settings groups.
-- ============================================================

-- ── Update venue settings to include footer lines & logo ──────
UPDATE settings
SET value = jsonb_set(
    jsonb_set(
        jsonb_set(
            value::jsonb,
            '{footer_line_1}',
            '"Terima kasih!"',
            true
        ),
        '{footer_line_2}',
        '"TiketKu POS & Ticketing"',
        true
    ),
    '{logo}',
    'null',
    true
)::text
WHERE key = 'venue';

-- ── Add notification settings group ───────────────────────────
INSERT INTO settings (key, value) VALUES
    ('notification', '{
        "low_quota_threshold": 10,
        "notify_on_refund": true,
        "notify_on_scan_fail": true,
        "daily_report_enabled": false,
        "daily_report_time": "08:00"
    }')
ON CONFLICT (key) DO NOTHING;

-- ── Add payment settings group ─────────────────────────────────
INSERT INTO settings (key, value) VALUES
    ('payment', '{
        "qris_enabled": true,
        "cash_enabled": true,
        "transfer_enabled": true,
        "midtrans_merchant_id": "",
        "midtrans_server_key": "",
        "midtrans_is_production": false
    }')
ON CONFLICT (key) DO NOTHING;
