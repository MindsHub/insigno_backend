-- Table: public.marker

CREATE TABLE IF NOT EXISTS public.markers_reports
(
    id BIGSERIAL NOT NULL,
    user_f BIGINT NOT NULL,
    reported_marker BIGINT,
    CONSTRAINT report_marker_id_pkey PRIMARY KEY (id),
    CONSTRAINT reports_marker_from_fkey FOREIGN KEY (user_f)
        REFERENCES public.users (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE CASCADE,
    CONSTRAINT marker_created_by_fkey FOREIGN KEY (reported_marker)
        REFERENCES public.markers (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE CASCADE
)

TABLESPACE pg_default;

ALTER TABLE IF EXISTS public.markers_reports
    OWNER to mindshub;
/*
CREATE OR REPLACE FUNCTION is_appropriate(inp_user_id BIGINT, pt FLOAT, res_date timestamp with time zone) RETURNS bool AS $$
		DECLARE ret BIGINT;
	BEGIN
		
	END;
$$ LANGUAGE plpgsql;*/