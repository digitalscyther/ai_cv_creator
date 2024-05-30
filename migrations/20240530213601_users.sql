CREATE TABLE IF NOT EXISTS "users" (
    id SERIAL PRIMARY KEY,
    profession TEXT,
    questions JSONB,
    resume TEXT,
    messages JSONB NOT NULL DEFAULT '[]'::JSONB,
    tokens_spent INT NOT NULL DEFAULT 0
);
