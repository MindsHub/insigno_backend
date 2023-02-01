-- Your SQL goes here
CREATE TABLE IF NOT EXISTS public.pills (
  id SERIAL PRIMARY KEY,
  text TEXT NOT NULL,
  author TEXT NOT NULL,
  source TEXT NOT NULL
)

--text: String,
--  author: String,
--  source: String,