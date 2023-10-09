-- Your SQL goes here

CREATE OR REPLACE FUNCTION verify_set_verdict(
    user_id_inp BIGINT,
    image_id_inp BIGINT,
    verdict_inp BOOLEAN
) RETURNS BOOLEAN AS $$
    DECLARE session_id BIGINT;
    DECLARE remaining_images INT;

    BEGIN
		IF now() < time_to_verify(user_id_inp) THEN
			RAISE EXCEPTION 'cant_verify_now';
		END IF;

        UPDATE image_verifications
        SET verdict = verdict_inp
        FROM verification_sessions
        WHERE image_verifications.image_id = image_id_inp
            AND image_verifications.verification_session = verification_sessions.id
            AND verification_sessions.completion_date IS NULL
            AND verification_sessions.user_id = user_id_inp
        RETURNING verification_sessions.id
        INTO session_id;

        IF session_id IS NULL THEN
            RAISE EXCEPTION 'session_not_found';
        END IF;

        SELECT COUNT(*)
        FROM image_verifications
        WHERE image_verifications.verification_session = session_id
            AND image_verifications.verdict IS NULL
        INTO remaining_images;

        IF remaining_images <> 0 THEN
            RETURN FALSE;
        END IF;

        UPDATE users
        SET last_revision = NOW()
        WHERE users.id = user_id_inp;

        UPDATE verification_sessions
        SET completion_date = NOW()
        WHERE verification_sessions.id = session_id;

        RETURN TRUE;
    END;
$$ LANGUAGE plpgsql;

-- UPDATE users SET last_revision = TIMESTAMP '2023-04-01 12:00:00+01';
-- UPDATE image_verifications SET verdict = NULL;
-- SELECT * FROM verify_set_verdict(1,1,TRUE);
-- SELECT * FROM verify_set_verdict(1,2,TRUE);
-- SELECT * FROM verify_set_verdict(1,3,TRUE);
-- SELECT * FROM verify_set_verdict(1,4,TRUE);
-- SELECT * FROM verify_set_verdict(1,5,TRUE);
-- SELECT * FROM verify_set_verdict(1,6,TRUE);
-- SELECT * FROM verify_set_verdict(1,6,TRUE);
-- SELECT * FROM verify_set_verdict(1,7,TRUE);
-- SELECT * FROM verify_set_verdict(1,7,TRUE);
-- DROP FUNCTION verify_set_verdict;