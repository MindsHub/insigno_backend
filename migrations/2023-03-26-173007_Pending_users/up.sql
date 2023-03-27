
CREATE TABLE IF NOT EXISTS public.pending_users
(
    id BIGSERIAL NOT NULL,
    name character varying(254) COLLATE pg_catalog."default" NOT NULL,
    email character varying(254) COLLATE pg_catalog."default",
    password_hash character varying(255) COLLATE pg_catalog."default" NOT NULL,
    request_date timestamp with time zone DEFAULT now(),
    token character varying(254) COLLATE pg_catalog."default" UNIQUE,
    CONSTRAINT pending_users_id_pkey PRIMARY KEY (id)
)

TABLESPACE pg_default;

ALTER TABLE IF EXISTS public.pending_users
    OWNER to mindshub;

CREATE OR REPLACE FUNCTION get_pending_user(token TEXT) RETURNS pending_users AS $$
		DECLARE ret pending_users;
	BEGIN
		DELETE 
            FROM pending_users 
            WHERE request_date+'1h'<now();
        SELECT * 
        FROM pending_users 
        WHERE pending_users.token=$1
        INTO ret;
        return ret;
	END;
$$ LANGUAGE plpgsql;