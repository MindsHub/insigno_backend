-- Your SQL goes here
ALTER TABLE public.users
ADD is_adult boolean DEFAULT false;

ALTER TABLE public.users
ALTER COLUMN is_adult SET NOT NULL;
