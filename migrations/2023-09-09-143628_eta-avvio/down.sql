-- This file should undo anything in `up.sql`
ALTER TABLE public.users
REMOVE is_adult;

ALTER TABLE public.users
REMOVE last_revision;