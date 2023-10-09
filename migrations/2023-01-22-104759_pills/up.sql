-- Your SQL goes here
CREATE TABLE IF NOT EXISTS public.pills (
  id BIGSERIAL PRIMARY KEY,
  text TEXT NOT NULL,
  author TEXT NOT NULL,
  source TEXT NOT NULL,
  accepted BOOLEAN NOT NULL
)
ALTER TABLE IF EXISTS public.pills OWNER TO mindshub;
