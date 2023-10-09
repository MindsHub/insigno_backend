-- Your SQL goes here
CREATE TABLE IF NOT EXISTS public.user_sessions
(
    user_id BIGSERIAL NOT NULL,
    token character varying(254) COLLATE pg_catalog."default" NOT NULL,
    refresh_date  timestamp with time zone NOT NULL,
    CONSTRAINT users_id_pkey PRIMARY KEY (user_id),
    CONSTRAINT users_id_fkey FOREIGN KEY (user_id)
        REFERENCES public.users (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE cascade
);
ALTER TABLE IF EXISTS public.user_sessions OWNER TO mindshub;


CREATE OR REPLACE FUNCTION is_session_valid(inp_date TIMESTAMP WITH TIME ZONE) RETURNS BOOL AS $$
	BEGIN
		return inp_date > now() - interval '7 days';
	END;
$$ LANGUAGE plpgsql;


CREATE OR REPLACE FUNCTION autenticate(id_inp BIGINT, tok TEXT) RETURNS users AS $$
	DECLARE ret users;
	DECLARE ret_row BigInt;
	BEGIN
		--check if token is valid
		PERFORM *
		FROM user_sessions
		WHERE is_session_valid(refresh_date) AND id_inp=user_id AND token = tok;
		GET diagnostics ret_row = row_count;
		IF(ret_row=0) THEN
			RAISE EXCEPTION 'token_invalid';
		END IF;

		-- refresh token
		UPDATE user_sessions
		SET refresh_date=now()
		WHERE user_id=id_inp;

		SELECT *
		FROM users
		WHERE users.id=id_inp
		INTO ret;
		RETURN ret;

	END;
$$ LANGUAGE plpgsql;
--SELECT * FROM autenticate(1, 'test');

