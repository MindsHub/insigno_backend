-- This file should undo anything in `up.sql`
drop FUNCTION IF EXISTS get_pending;
DROP TABLE IF EXISTS public.pending;
