-- Your SQL goes here
ALTER TABLE IF EXISTS marker_images
ADD created_by BIGINT DEFAULT NULL;

UPDATE marker_images
SET created_by = markers.created_by
FROM markers
WHERE markers.id=marker_images.refers_to;

ALTER TABLE marker_images ALTER COLUMN created_by SET NOT NULL;

CREATE OR REPLACE FUNCTION public.create_verify(
	user_id_inp bigint)
    RETURNS TABLE(id bigint, verification_session bigint, image_id bigint, marker_id bigint, verdict boolean) 
    LANGUAGE 'plpgsql'
    COST 100
    VOLATILE PARALLEL UNSAFE
    ROWS 1000

AS $BODY$
	#variable_conflict use_column
	DECLARE session_id BIGINT;
	DECLARE to_choose BIGINT;

	BEGIN
		INSERT INTO verification_sessions(user_id) VALUES (user_id_inp) RETURNING id INTO session_id;

		SELECT ceil(log(2, count(marker_images.id)+1))+5
			FROM marker_images
			WHERE verify_number<3
			INTO to_choose;

		INSERT INTO image_verifications(verification_session, image_id, marker_id)
			SELECT session_id, id, refers_to
				FROM marker_images
				WHERE created_by<>user_id_inp
				ORDER BY verify_number ASC,
				random()
				LIMIT to_choose;
		RETURN QUERY
			SELECT *
				FROM image_verifications
				WHERE verification_session = session_id;
	END;
$BODY$;