
DROP FUNCTION IF EXISTS resolve_marker;
CREATE OR REPLACE FUNCTION resolve_marker(marker_id BIGINT, user_id BIGINT) RETURNS BOOL AS $$
	DECLARE ret BIGINT;
	DECLARE adesso timestamp with time zone = NOW();
	BEGIN
		UPDATE markers
		SET resolution_date= adesso
		WHERE id=marker_id AND resolution_date is NULL;
		SELECT COUNT (*)
		FROM MARKERS
		INTO ret
		WHERE id=marker_id AND resolution_date = adesso;
		if (ret>0) THEN
			RETURN FALSE;
		END IF;
		RETURN TRUE;
	END;
$$ LANGUAGE plpgsql;
    

SELECT * FROM resolve_marker(1, 1);