-- Table: public.marker_reports

CREATE TABLE IF NOT EXISTS public.marker_reports
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
ALTER TABLE IF EXISTS public.marker_reports OWNER TO mindshub;