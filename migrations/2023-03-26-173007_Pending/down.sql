-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS public.pending;
drop FUNCTION IF EXISTS get_pending;