-- Table: public.marker

-- DROP TABLE IF EXISTS public.marker;

CREATE TABLE IF NOT EXISTS public.marker_images
(
    id BIGSERIAL NOT NULL,
    path text NOT NULL,
    refers_to BIGINT NOT NULL,
    CONSTRAINT marker_images_id_pkey PRIMARY KEY (id),
    CONSTRAINT marker_images_refers_to_fkey FOREIGN KEY (refers_to)
        REFERENCES public.markers (id) MATCH SIMPLE
        ON UPDATE cascade
        ON DELETE cascade
)

TABLESPACE pg_default;

ALTER TABLE IF EXISTS public.marker_images
    OWNER to mindshub;
