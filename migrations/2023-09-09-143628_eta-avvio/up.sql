-- Your SQL goes here
ALTER TABLE public.users
ADD is_adult boolean DEFAULT false;

ALTER TABLE public.users
ADD last_revision timestamp with time zone DEFAULT TIMESTAMP '2023-04-01 12:00:00+01' NOT NULL;

ALTER TABLE public.users
ALTER COLUMN is_adult SET NOT NULL;