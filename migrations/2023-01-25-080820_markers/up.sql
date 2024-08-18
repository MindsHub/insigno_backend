-- Table: public.marker

CREATE TABLE IF NOT EXISTS public.markers
(
    id BIGSERIAL NOT NULL,
    point geometry(Geometry, 4326) NOT NULL,
    creation_date timestamp with time zone DEFAULT now(),
    resolution_date timestamp with time zone DEFAULT NULL,
    marker_types_id BIGINT NOT NULL DEFAULT '1',
    created_by BIGINT NOT NULL,
    solved_by BIGINT,
    CONSTRAINT marker_id_pkey PRIMARY KEY (id),
    CONSTRAINT marker_id_fkey FOREIGN KEY (marker_types_id)
        REFERENCES public.marker_types (id) MATCH SIMPLE
        ON UPDATE cascade
        ON DELETE NO ACTION,
    CONSTRAINT marker_created_by_fkey FOREIGN KEY (created_by)
        REFERENCES public.users (id) MATCH SIMPLE
        ON UPDATE cascade
        ON DELETE NO ACTION
);

--INSERT INTO public.markers(
--	id, point, creation_date)
--	VALUES
--  (1, ST_GeomFromText('POINT(11.003296 45.755445)', 4326), '2014-06-04 12:00 EDT');