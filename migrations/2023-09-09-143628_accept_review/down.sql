-- This file should undo anything in `up.sql`
ALTER TABLE public.users
DROP IF EXISTS accepted_to_review;