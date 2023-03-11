-- This file should undo anything in `up.sql`
--DELETE resolve_marker if exists ;
DROP FUNCTION IF EXISTS resolve_marker;
DROP FUNCTION IF EXISTS assign_point;
DROP FUNCTION IF EXISTS _marker_update;