-- This file should undo anything in `up.sql`
ALTER TABLE public.users
REMOVE accepted_to_review;