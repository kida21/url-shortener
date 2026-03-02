CREATE TABLE IF NOT EXISTS urls (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    short_code   TEXT NOT NULL UNIQUE,
    original_url TEXT NOT NULL,
    click_count  INTEGER DEFAULT 0,
    created_at   TEXT DEFAULT (datetime('now')),
    expires_at   TEXT NULL
);

CREATE INDEX IF NOT EXISTS idx_short_code ON urls(short_code);
CREATE INDEX IF NOT EXISTS idx_original_url ON urls(original_url);