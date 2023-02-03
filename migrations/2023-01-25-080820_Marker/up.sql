-- Table: public.marker

-- DROP TABLE IF EXISTS public.marker;

CREATE TABLE IF NOT EXISTS public.marker
(
    id integer NOT NULL,
    point geometry(Geometry, 4326) NOT NULL,
    creation_date timestamp with time zone NOT NULL,
    trash_type_id integer NOT NULL DEFAULT '1',
    CONSTRAINT marker_id_pkey PRIMARY KEY (id),
    CONSTRAINT marker_id_fkey FOREIGN KEY (trash_type_id)
        REFERENCES public.trash_type (id) MATCH SIMPLE
        ON UPDATE cascade
        ON DELETE NO ACTION
)

TABLESPACE pg_default;

ALTER TABLE IF EXISTS public.marker
    OWNER to mindshub;

INSERT INTO public.marker(
	id, point, creation_date)
	VALUES 
  (1, ST_GeomFromText('POINT(11.003296 45.755445)', 4326), '2014-06-04 12:00 EDT');