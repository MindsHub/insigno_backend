-- Your SQL goes here

ALTER TABLE public.users
ADD last_revision timestamp with time zone DEFAULT TIMESTAMP '2023-04-01 12:00:00+01' NOT NULL;
ALTER TABLE public.marker_images
ADD verdict_number BIGINT DEFAULT 0 NOT NULL;
ALTER TABLE public.marker_images
ADD avarage_verdict FLOAT DEFAULT 0.0 NOT NULL;


CREATE TABLE IF NOT EXISTS public.verification_sessions
(
    id BIGSERIAL,
	user_id BIGINT NOT NULL,
	completion_date timestamp with time zone,

    CONSTRAINT verification_sessions_id PRIMARY KEY (id),
	CONSTRAINT user_id_fkey FOREIGN KEY (user_id)
        REFERENCES public.users (id) MATCH SIMPLE
        ON UPDATE cascade
        ON DELETE NO ACTION
);
ALTER TABLE IF EXISTS public.verification_sessions OWNER TO mindshub;

CREATE TABLE IF NOT EXISTS public.image_verifications
(
    id BIGSERIAL,
	verification_session BIGINT,
	image_id BIGINT NOT NULL,
	marker_id BIGINT NOT NULL,
	verdict BOOLEAN,

    CONSTRAINT image_verifications_id PRIMARY KEY (id),
	CONSTRAINT verification_session_id_fkey FOREIGN KEY (verification_session)
        REFERENCES public.verification_sessions (id) MATCH SIMPLE
        ON UPDATE cascade
        ON DELETE NO ACTION,
	CONSTRAINT image_id_fkey FOREIGN KEY (image_id)
        REFERENCES public.marker_images (id) MATCH SIMPLE
        ON UPDATE cascade
        ON DELETE cascade,
	CONSTRAINT marker_id_fkey FOREIGN KEY (marker_id)
        REFERENCES public.markers (id) MATCH SIMPLE
        ON UPDATE cascade
        ON DELETE cascade
);
ALTER TABLE IF EXISTS public.image_verifications OWNER TO mindshub;


CREATE OR REPLACE FUNCTION time_to_verify(user_id BIGINT) RETURNS timestamp AS $$
	DECLARE ret timestamp ;
	BEGIN
        SELECT (users.last_revision+interval '8 hours') AT TIME ZONE 'UTC'
			FROM users
			WHERE users.id=user_id
		    INTO ret;
		RETURN ret;
	END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION get_to_verify(user_id_inp BIGINT) RETURNS TABLE(
	image_id BIGINT,
	marker_id BIGINT,
	verdict BOOLEAN,
	marker_types_id BIGINT,
	all_marker_images BIGINT[]
) AS $$
	#variable_conflict use_column
	DECLARE ret BIGINT;
	BEGIN
		IF now() < time_to_verify(user_id_inp) THEN
			RAISE EXCEPTION 'cant_verify_now';
		END IF;

		SELECT id
		FROM public.verification_sessions
		WHERE user_id = user_id_inp AND
			completion_date IS NULL
		INTO ret;

		if ret is NULL THEN
			--create a new one
			RETURN QUERY
				SELECT
					image_verifications.image_id,
					image_verifications.marker_id,
					image_verifications.verdict,
					markers.marker_types_id,
					ARRAY_AGG(marker_images.id) AS all_marker_images
				FROM create_verify(user_id_inp) AS image_verifications
				JOIN markers ON markers.id = image_verifications.marker_id
				JOIN marker_images ON marker_images.refers_to = markers.id
				GROUP BY
					image_verifications.id,
					image_verifications.image_id,
					image_verifications.marker_id,
					image_verifications.verdict,
					markers.marker_types_id;
		ELSE
		-- return the first
			RETURN QUERY
				SELECT
					image_verifications.image_id,
					image_verifications.marker_id,
					image_verifications.verdict,
					markers.marker_types_id,
					ARRAY_AGG(marker_images.id) AS all_marker_images
				FROM image_verifications
				JOIN markers ON markers.id = image_verifications.marker_id
				JOIN marker_images ON marker_images.refers_to = markers.id
				WHERE verification_session = ret
				GROUP BY
					image_verifications.id,
					image_verifications.image_id,
					image_verifications.marker_id,
					image_verifications.verdict,
					markers.marker_types_id;
		END IF;
	END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION create_verify(user_id_inp BIGINT) RETURNS TABLE(
	id BIGINT,
	verification_session BIGINT,
	image_id BIGINT,
	marker_id BIGINT,
	verdict BOOLEAN
) language plpgsql AS $$
	#variable_conflict use_column
	DECLARE session_id BIGINT;
	DECLARE to_choose BIGINT;

	BEGIN
		INSERT INTO verification_sessions(user_id) VALUES (user_id_inp) RETURNING id INTO session_id;

		SELECT ceil(log(2, count(marker_images.id)+1))+5
			FROM marker_images
			WHERE verdict_number<3
			INTO to_choose;

		INSERT INTO image_verifications(verification_session, image_id, marker_id)
			SELECT session_id, id, refers_to
				FROM marker_images
				--WHERE user_id_inp<>user_id
				ORDER BY verdict_number ASC,
				random()
				LIMIT to_choose;
		RETURN QUERY
			SELECT *
				FROM image_verifications
				WHERE verification_session = session_id;
	END;
$$;

--SELECT * FROM add_marker(1,ST_Point( -71.104, 42.315) , 1);
--INSERT INTO marker_images(path, refers_to, verdict_number) VALUES('', 1, 2);

--SELECT * FROM get_to_verify(1);