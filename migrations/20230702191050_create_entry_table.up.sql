CREATE TABLE entries(
    id TEXT PRIMARY KEY,
    channel_id INT NOT NULL,
    title TEXT NOT NULL,
    link TEXT NOT NULL,
    published TIMESTAMPTZ NOT NULL,
    updated TIMESTAMPTZ NOT NULL,
    read BOOLEAN NOT NULL DEFAULT 'true',
    CONSTRAINT fk_channel FOREIGN KEY(channel_id) REFERENCES channels(id)
)