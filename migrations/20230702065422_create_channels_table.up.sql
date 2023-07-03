CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE channels(
    id UUID PRIMARY KEY NOT NULL DEFAULT (uuid_generate_v4()),
    link TEXT NOT NULL
)