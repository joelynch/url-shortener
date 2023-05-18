CREATE TABLE IF NOT EXISTS urls (
    id SERIAL PRIMARY KEY,
    code TEXT NOT NULL,
    url TEXT NOT NULL,
    short_url TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE (code)
);

CREATE TABLE IF NOT EXISTS hits (
    id SERIAL PRIMARY KEY,
    url_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    FOREIGN KEY (url_id) REFERENCES urls(id)
);

CREATE INDEX IF NOT EXISTS hits_url_id_idx ON hits (url_id);
