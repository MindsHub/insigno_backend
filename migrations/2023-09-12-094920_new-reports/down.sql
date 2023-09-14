-- This file should undo anything in `up.sql`

ALTER TABLE public.users
REMOVE last_revision;

REMOVE FUNCTION time_to_publish;
REMOVE FUNCTION published;
