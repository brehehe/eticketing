-- ============================================================
-- TiketKu — Initial Schema
-- ============================================================

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ── Users & Auth ──────────────────────────────────────────────
CREATE TABLE users (
    id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    name           TEXT        NOT NULL,
    username       TEXT        NOT NULL UNIQUE,
    email          TEXT        NOT NULL UNIQUE,
    password_hash  TEXT        NOT NULL,
    role           TEXT        NOT NULL DEFAULT 'kasir'
                   CHECK (role IN ('admin','kasir','officer','investor')),
    status         TEXT        NOT NULL DEFAULT 'active'
                   CHECK (status IN ('active','inactive')),
    avatar         TEXT,
    totp_secret    TEXT,
    totp_enabled   BOOLEAN     NOT NULL DEFAULT FALSE,
    last_login     TIMESTAMPTZ,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email    ON users(email);
CREATE INDEX idx_users_role     ON users(role);

-- Device tracking for multi-login control
CREATE TABLE user_devices (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id    TEXT        NOT NULL,
    device_info  TEXT,
    ip_address   INET,
    last_seen    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, device_id)
);

-- Password reset tokens
CREATE TABLE password_resets (
    token        TEXT        PRIMARY KEY,
    user_id      UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at   TIMESTAMPTZ NOT NULL,
    used         BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Investors ─────────────────────────────────────────────────
CREATE TABLE investors (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID        REFERENCES users(id) ON DELETE SET NULL,
    name         TEXT        NOT NULL,
    email        TEXT,
    phone        TEXT,
    status       TEXT        NOT NULL DEFAULT 'active',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Categories ────────────────────────────────────────────────
CREATE TABLE categories (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    name         TEXT        NOT NULL,
    slug         TEXT        NOT NULL UNIQUE,
    icon         TEXT,
    sort_order   INT         NOT NULL DEFAULT 0,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO categories (name, slug, icon, sort_order) VALUES
    ('Wahana',   'wahana',   'Ticket',    1),
    ('Makanan',  'makanan',  'UtensilsCrossed', 2),
    ('Minuman',  'minuman',  'Coffee',    3),
    ('Souvenir', 'souvenir', 'ShoppingBag', 4);

-- ── Products ──────────────────────────────────────────────────
CREATE TABLE products (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT        NOT NULL,
    sku             TEXT        NOT NULL UNIQUE,
    category_id     UUID        REFERENCES categories(id) ON DELETE SET NULL,
    price           BIGINT      NOT NULL DEFAULT 0,
    investor_price  BIGINT      NOT NULL DEFAULT 0,
    investor_id     UUID        REFERENCES investors(id) ON DELETE SET NULL,
    quota           INT,
    quota_used      INT         NOT NULL DEFAULT 0,
    schedule        TEXT,
    status          TEXT        NOT NULL DEFAULT 'active'
                    CHECK (status IN ('active','inactive','draft')),
    has_variants    BOOLEAN     NOT NULL DEFAULT FALSE,
    ticket_required BOOLEAN     NOT NULL DEFAULT TRUE,
    print_enabled   BOOLEAN     NOT NULL DEFAULT TRUE,
    scan_enabled    BOOLEAN     NOT NULL DEFAULT TRUE,
    barcode         TEXT,
    image           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_products_status   ON products(status);
CREATE INDEX idx_products_category ON products(category_id);
CREATE INDEX idx_products_sku      ON products(sku);

CREATE TABLE product_variants (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id  UUID        NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    name        TEXT        NOT NULL,
    sku         TEXT        NOT NULL UNIQUE,
    price       BIGINT      NOT NULL,
    investor_price BIGINT   NOT NULL DEFAULT 0,
    quota       INT,
    quota_used  INT         NOT NULL DEFAULT 0,
    status      TEXT        NOT NULL DEFAULT 'active',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_variants_product ON product_variants(product_id);

-- Investor-product mapping
CREATE TABLE investor_products (
    id          UUID    PRIMARY KEY DEFAULT gen_random_uuid(),
    investor_id UUID    NOT NULL REFERENCES investors(id) ON DELETE CASCADE,
    product_id  UUID    NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    revenue_pct NUMERIC(5,2) NOT NULL DEFAULT 0,
    UNIQUE(investor_id, product_id)
);

-- ── Payment Methods ───────────────────────────────────────────
CREATE TABLE payment_methods (
    id              UUID    PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT    NOT NULL,
    provider        TEXT,
    fee_pct         NUMERIC(5,2) NOT NULL DEFAULT 0,
    fee_flat        BIGINT  NOT NULL DEFAULT 0,
    auto_settlement BOOLEAN NOT NULL DEFAULT TRUE,
    status          TEXT    NOT NULL DEFAULT 'active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO payment_methods (name, provider, fee_pct, auto_settlement) VALUES
    ('Tunai',        NULL,        0,    TRUE),
    ('QRIS',         'Midtrans',  0.7,  TRUE),
    ('Transfer Bank','BCA',       0,    FALSE),
    ('Debit',        'BCA',       0,    TRUE),
    ('E-Wallet',     'GoPay',     0.5,  TRUE);

-- ── Vouchers ──────────────────────────────────────────────────
CREATE TABLE vouchers (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    code            TEXT        NOT NULL UNIQUE,
    type            TEXT        NOT NULL CHECK (type IN ('nominal','persen')),
    value           BIGINT      NOT NULL,
    min_purchase    BIGINT      NOT NULL DEFAULT 0,
    max_discount    BIGINT,
    quota           INT,
    quota_used      INT         NOT NULL DEFAULT 0,
    valid_from      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_until     TIMESTAMPTZ,
    status          TEXT        NOT NULL DEFAULT 'active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Transactions ──────────────────────────────────────────────
CREATE TABLE transactions (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice      TEXT        NOT NULL UNIQUE,
    cashier_id   UUID        NOT NULL REFERENCES users(id),
    voucher_id   UUID        REFERENCES vouchers(id) ON DELETE SET NULL,
    status       TEXT        NOT NULL DEFAULT 'pending'
                 CHECK (status IN ('pending','paid','refunded','cancelled')),
    subtotal     BIGINT      NOT NULL DEFAULT 0,
    discount     BIGINT      NOT NULL DEFAULT 0,
    total        BIGINT      NOT NULL DEFAULT 0,
    change       BIGINT      NOT NULL DEFAULT 0,
    note         TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    paid_at      TIMESTAMPTZ,
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_transactions_cashier    ON transactions(cashier_id);
CREATE INDEX idx_transactions_status     ON transactions(status);
CREATE INDEX idx_transactions_created_at ON transactions(created_at DESC);
CREATE INDEX idx_transactions_invoice    ON transactions(invoice);

CREATE TABLE transaction_items (
    id             UUID    PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_id UUID    NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    product_id     UUID    NOT NULL REFERENCES products(id),
    variant_id     UUID    REFERENCES product_variants(id),
    qty            INT     NOT NULL DEFAULT 1,
    unit_price     BIGINT  NOT NULL,
    discount       BIGINT  NOT NULL DEFAULT 0,
    subtotal       BIGINT  NOT NULL
);

CREATE INDEX idx_tx_items_transaction ON transaction_items(transaction_id);
CREATE INDEX idx_tx_items_product     ON transaction_items(product_id);

CREATE TABLE payments (
    id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_id UUID        NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    method         TEXT        NOT NULL,
    provider       TEXT,
    amount         BIGINT      NOT NULL,
    fee            BIGINT      NOT NULL DEFAULT 0,
    reference      TEXT,
    status         TEXT        NOT NULL DEFAULT 'pending'
                   CHECK (status IN ('pending','paid','failed','refunded')),
    paid_at        TIMESTAMPTZ,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_payments_transaction ON payments(transaction_id);

-- ── Tickets ───────────────────────────────────────────────────
CREATE TABLE tickets (
    id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    code           TEXT        NOT NULL UNIQUE,
    qr_data        TEXT        NOT NULL,
    transaction_id UUID        NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    product_id     UUID        NOT NULL REFERENCES products(id),
    variant_id     UUID        REFERENCES product_variants(id),
    visitor_name   TEXT,
    queue_number   INT         NOT NULL DEFAULT 1,
    status         TEXT        NOT NULL DEFAULT 'active'
                   CHECK (status IN ('active','used','expired','refunded')),
    valid_until    TIMESTAMPTZ,
    used_at        TIMESTAMPTZ,
    used_by        UUID        REFERENCES users(id) ON DELETE SET NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tickets_code        ON tickets(code);
CREATE INDEX idx_tickets_transaction ON tickets(transaction_id);
CREATE INDEX idx_tickets_status      ON tickets(status);

CREATE TABLE ticket_scans (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_id    UUID        NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    officer_id   UUID        NOT NULL REFERENCES users(id),
    scan_method  TEXT        NOT NULL DEFAULT 'manual'
                 CHECK (scan_method IN ('camera','bluetooth','manual')),
    result       TEXT        NOT NULL
                 CHECK (result IN ('valid','invalid','expired','already_used')),
    note         TEXT,
    scanned_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_scans_ticket     ON ticket_scans(ticket_id);
CREATE INDEX idx_scans_officer    ON ticket_scans(officer_id);
CREATE INDEX idx_scans_scanned_at ON ticket_scans(scanned_at DESC);

-- ── Bluetooth Devices ─────────────────────────────────────────
CREATE TABLE bluetooth_devices (
    id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    name           TEXT        NOT NULL,
    address        TEXT        NOT NULL UNIQUE,
    type           TEXT        NOT NULL DEFAULT 'printer'
                   CHECK (type IN ('printer','scanner')),
    is_default     BOOLEAN     NOT NULL DEFAULT FALSE,
    last_connected TIMESTAMPTZ,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Settings ──────────────────────────────────────────────────
CREATE TABLE settings (
    key         TEXT        PRIMARY KEY,
    value       TEXT        NOT NULL DEFAULT '{}',
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO settings (key, value) VALUES
    ('venue',   '{"name":"","address":"","phone":"","email":"","operating_hours":""}'),
    ('ticket',  '{"print_enabled":true,"qr_enabled":true,"expiry_hours":24,"ticket_prefix":"TK"}'),
    ('printer', '{"auto_retry":true,"retry_count":3,"paper_width":"80mm"}'),
    ('shift',   '{"shift_enabled":false,"auto_closing":false,"opening_balance":0}'),
    ('system',  '{"theme":"system","session_timeout_min":480,"cache_ttl_sec":300}');

-- ── Audit Log ─────────────────────────────────────────────────
CREATE TABLE audit_logs (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID        REFERENCES users(id) ON DELETE SET NULL,
    action      TEXT        NOT NULL,
    entity      TEXT        NOT NULL,
    entity_id   TEXT,
    old_data    JSONB,
    new_data    JSONB,
    ip_address  INET,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_user      ON audit_logs(user_id);
CREATE INDEX idx_audit_entity    ON audit_logs(entity, entity_id);
CREATE INDEX idx_audit_created   ON audit_logs(created_at DESC);

-- ── Seed: default admin user ───────────────────────────────────
-- Password: admin123 (bcrypt cost 12)
INSERT INTO users (name, username, email, password_hash, role, status) VALUES
    ('Administrator', 'admin', 'admin@tiketku.id',
     '$2b$12$pe5rYKNa69medTPxzth//.io5bX6lMhEZGb/pkVXdag/EwhE0HVOm',
     'admin', 'active')
ON CONFLICT (username) DO NOTHING;
