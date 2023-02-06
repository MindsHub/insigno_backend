-- Table: public.marker

-- DROP TABLE IF EXISTS public.marker;

CREATE TABLE IF NOT EXISTS public.marker_images
(
    id SERIAL NOT NULL,
    path text NOT NULL,
    refers_to integer NOT NULL,
    CONSTRAINT marker_images_id_pkey PRIMARY KEY (id),
    CONSTRAINT marker_images_refers_to_fkey FOREIGN KEY (refers_to)
        REFERENCES public.markers (id) MATCH SIMPLE
        ON UPDATE cascade
        ON DELETE NO ACTION
)

TABLESPACE pg_default;

ALTER TABLE IF EXISTS public.marker_images
    OWNER to mindshub;

--INSERT INTO public.markers(
--	id, point, creation_date)
--	VALUES 
--  (1, ST_GeomFromText('POINT(11.003296 45.755445)', 4326), '2014-06-04 12:00 EDT');