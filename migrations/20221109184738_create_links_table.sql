-- Create Links Table
CREATE TABLE links(
    id uuid NOT NULL,
    PRIMARY KEY(id),
    url TEXT NOT NULL,
    key TEXT NOT NULL,
    created_at timestamptz NOT NULL
)