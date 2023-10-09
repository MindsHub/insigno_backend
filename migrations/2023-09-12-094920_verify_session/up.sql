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
	completition_date timestamp with time zone,

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
	verdict BOOLEAN,

    CONSTRAINT image_verifications_id PRIMARY KEY (id),
	CONSTRAINT verification_session_id_fkey FOREIGN KEY (verification_session)
        REFERENCES public.verification_sessions (id) MATCH SIMPLE
        ON UPDATE cascade
        ON DELETE NO ACTION,
	CONSTRAINT image_id_fkey FOREIGN KEY (image_id)
        REFERENCES public.marker_images (id) MATCH SIMPLE
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

CREATE OR REPLACE FUNCTION get_to_verify(user_id_inp BIGINT) RETURNS TABLE( id BIGINT, verification_session BIGINT, image_id BIGINT, verdict BOOLEAN) AS $$
	#variable_conflict use_column
	DECLARE ret BIGINT;
	BEGIN
		IF now() < time_to_verify(user_id_inp) THEN
			RAISE EXCEPTION 'you cant verify right now';
		END IF;

		SELECT id
		FROM public.verification_sessions
		WHERE user_id = user_id_inp AND
			completition_date IS NULL
		INTO ret;

		if ret is NULL THEN
			--create a new one
			RETURN query
				SELECT * FROM create_verify(user_id_inp);
		ELSE
		-- return the first
			RETURN query
				SELECT *
					FROM image_verifications
					WHERE verification_session=ret;
		END IF;
	END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION create_verify(user_id_inp BIGINT) RETURNS TABLE( id BIGINT, verification_session BIGINT, image_id BIGINT, verdict BOOLEAN) language plpgsql AS $$
	#variable_conflict use_column
	DECLARE session_id BIGINT;
	DECLARE to_choose BIGINT;

	BEGIN
	INSERT INTO verification_sessions(user_id) VALUES (user_id_inp) RETURNING id INTO session_id;

	SELECT ceil(log(2, count(marker_images.id)+1))+5
		FROM marker_images
		WHERE verdict_number<3
		INTO to_choose;

	INSERT INTO image_verifications(verification_session, image_id)
		SELECT session_id, id
			FROM marker_images
			--WHERE user_id_inp<>user_id
			ORDER BY verdict_number ASC,
			random()
			LIMIT to_choose;
	return query
		SELECT *
			FROM image_verifications
			WHERE verification_session=session_id;
	END;
$$;

--SELECT * FROM add_marker(1,ST_Point( -71.104, 42.315) , 1);
--INSERT INTO marker_images(path, refers_to, verdict_number) VALUES('', 1, 2);

--SELECT * FROM get_to_verify(1);