-- Your SQL goes here
CREATE TABLE IF NOT EXISTS public.pills (
  id BIGSERIAL PRIMARY KEY,
  text TEXT NOT NULL,
  author TEXT NOT NULL,
  source TEXT NOT NULL,
  accepted BOOLEAN NOT NULL
)

--text: String,
--  author: String,
--  source: String,