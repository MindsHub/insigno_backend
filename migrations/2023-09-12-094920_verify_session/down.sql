-- This file should undo anything in `up.sql`

DROP FUNCTION IF EXISTS time_to_verify;
DROP FUNCTION IF EXISTS get_to_verify;
DROP FUNCTION IF EXISTS create_verify;

ALTER TABLE public.users
DROP IF EXISTS last_revision;
ALTER TABLE public.marker_images
DROP IF EXISTS verify_number;
ALTER TABLE public.marker_images
DROP IF EXISTS verify_average;

DROP TABLE IF EXISTS public.image_verifications;
DROP TABLE IF EXISTS public.verification_sessions;