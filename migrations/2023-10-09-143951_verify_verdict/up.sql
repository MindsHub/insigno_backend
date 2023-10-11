-- Your SQL goes here

CREATE OR REPLACE FUNCTION verify_set_verdict(
    user_id_inp BIGINT,
    image_id_inp BIGINT,
    verdict_inp BOOLEAN
) RETURNS DOUBLE PRECISION AS $$
    DECLARE session_id BIGINT;
    DECLARE total_rewarded_points DOUBLE PRECISION;
    DECLARE remaining_images_count INT;

    BEGIN
        -- check if the user can verify
		IF now() < time_to_verify(user_id_inp) THEN
			RAISE EXCEPTION 'cant_verify_now';
		END IF;

        -- set the verdict to the image verification, and also check if
        -- the user has an active session containing the input image
        UPDATE image_verifications
        SET verdict = verdict_inp
        FROM verification_sessions
        WHERE image_verifications.image_id = image_id_inp
            AND image_verifications.verdict IS NULL
            AND image_verifications.verification_session = verification_sessions.id
            AND verification_sessions.completion_date IS NULL
            AND verification_sessions.user_id = user_id_inp
        RETURNING verification_sessions.id
        INTO session_id;

        -- if this exception is thrown, someone might be trying to tamper with the db
        IF session_id IS NULL THEN
            RAISE EXCEPTION 'session_not_found';
        END IF;

        -- update the score of the marker image
        UPDATE marker_images
        SET verify_number = verify_number + 1,
            verify_average = (verify_average * verify_number + CASE WHEN verdict_inp = TRUE THEN 1.0 ELSE 0.0 END)
        WHERE marker_images.id = image_id_inp;

        -- check if the remaining images in the session are down to 0
        SELECT 2.0 * COUNT(*),
            SUM(CASE WHEN image_verifications.verdict IS NULL THEN 1 ELSE 0 END)
        FROM image_verifications
        WHERE image_verifications.verification_session = session_id
        INTO total_rewarded_points, remaining_images_count;

        -- if there are some remaining images, return NULL as the number of points
        -- awarded, to signal that the session has not ended yet
        IF remaining_images_count <> 0 THEN
            RETURN NULL;
        END IF;

        -- the user completed this session,
        -- so set the last revision time and add some points
        UPDATE users
        SET last_revision = NOW(),
            points = points + total_rewarded_points
        WHERE users.id = user_id_inp;

        -- set the completion date on the verification session
        UPDATE verification_sessions
        SET completion_date = NOW()
        WHERE verification_sessions.id = session_id;

        -- return the number of awarded points
        RETURN total_rewarded_points;
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