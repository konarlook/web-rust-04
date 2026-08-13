-- User Tables
CREATE SCHEMA IF NOT EXISTS blog;
CREATE TABLE IF NOT EXISTS blog.users
(
    id            BIGSERIAL PRIMARY KEY,
    username      VARCHAR(64) UNIQUE,
    email         VARCHAR(256) UNIQUE,
    password_hash VARCHAR(256),
    created_at    TIMESTAMP WITH TIME ZONE
);
CREATE INDEX idx_username ON blog.users (username, email);

-- Post Tables
CREATE TABLE IF NOT EXISTS blog.posts
(
    id         BIGSERIAL PRIMARY KEY,
    title      VARCHAR(256),
    content    TEXT,
    author_id  BIGINT,
    created_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE,
    CONSTRAINT fk_author_id
        FOREIGN KEY (author_id)
            REFERENCES blog.users (id)
            ON DELETE CASCADE
);
CREATE INDEX idx_author_id ON blog.posts (author_id);
CREATE INDEX idx_created_at ON blog.posts (created_at);