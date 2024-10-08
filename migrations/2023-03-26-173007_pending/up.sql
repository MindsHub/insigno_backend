
CREATE TABLE IF NOT EXISTS public.pending
(
    id BIGSERIAL NOT NULL,
    token character varying(30) COLLATE pg_catalog."default" NOT NULL,
    action text COLLATE pg_catalog."default" NOT NULL,
    request_date timestamp with time zone DEFAULT now(),
    CONSTRAINT pending_id_pkey PRIMARY KEY (id)
);

CREATE OR REPLACE FUNCTION get_pending(token TEXT) RETURNS pending AS $$
		DECLARE ret pending;
	BEGIN
		DELETE
            FROM pending
            WHERE pending.request_date+'1h'<now();

        SELECT *
        FROM pending
        WHERE pending.token=$1
        INTO ret;

        DELETE FROM pending WHERE pending.token=$1;

        return ret;
	END;
$$ LANGUAGE plpgsql;