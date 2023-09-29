-- Your SQL goes here

--ALTER TABLE public.users
--ADD last_revision timestamp with time zone DEFAULT TIMESTAMP '2023-04-01 12:00:00+01' NOT NULL;

DELETE TABLE IF EXISTS public.verification_session;

CREATE TABLE IF NOT EXISTS public.verification_sessions
(
    id BIGSERIAL NOT NULL,
	user BIGINT NOT NULL,


    CONSTRAINT verification_sessions_id PRIMARY KEY (id)
	CONSTRAINT user_id_fkey FOREIGN KEY (user)
        REFERENCES public.user (id) MATCH SIMPLE
        ON UPDATE cascade
        ON DELETE NO ACTION,
);

CREATE TABLE IF NOT EXISTS public.image_verification
(
    id BIGSERIAL NOT NULL,
	verification_session BIGINT,


    CONSTRAINT image_verification_id PRIMARY KEY (id)
	CONSTRAINT verification_session_id_fkey FOREIGN KEY (verification_session)
        REFERENCES public.verification_session (id) MATCH SIMPLE
        ON UPDATE cascade
        ON DELETE NO ACTION,
);


ALTER TABLE IF EXISTS public.pending
    OWNER to mindshub;


CREATE OR REPLACE FUNCTION time_to_publish(user_id BIGINT) RETURNS interval AS $$
	DECLARE ret interval ;
	BEGIN
        SELECT (users.last_revision+interval '8 hours'-now())
			FROM users
			WHERE users.id=user_id
		    INTO ret;
		RETURN ret;
	END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION published(user_id BIGINT) RETURNS BIGINT AS $$
	DECLARE ret BIGINT;
	BEGIN
        UPDATE users SET last_revision=now(), points=points+10
			WHERE users.id=user_id;
			
		RETURN user_id;
	END;
$$ LANGUAGE plpgsql;

