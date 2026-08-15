-- User Tables
CREATE TABLE IF NOT EXISTS users
(
    id            BIGSERIAL PRIMARY KEY,
    username      VARCHAR(64) UNIQUE       NOT NULL,
    email         VARCHAR(256) UNIQUE      NOT NULL,
    password_hash VARCHAR(256)             NOT NULL,
    created_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);
CREATE INDEX idx_username ON users (username, email);

-- Post Tables
CREATE TABLE IF NOT EXISTS posts
(
    id         BIGSERIAL PRIMARY KEY,
    title      VARCHAR(256)             NOT NULL,
    content    TEXT,
    author_id  BIGINT                   NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
    CONSTRAINT fk_author_id
        FOREIGN KEY (author_id)
            REFERENCES users (id)
            ON DELETE CASCADE
);
CREATE INDEX idx_author_id ON posts (author_id);
CREATE INDEX idx_created_at ON posts (created_at);