-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS public.user_sessions;

DROP FUNCTION IF EXISTS is_session_valid;
DROP FUNCTION IF EXISTS autenticate;
