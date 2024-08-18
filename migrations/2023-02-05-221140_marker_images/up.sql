-- Table: public.marker_images

CREATE TABLE IF NOT EXISTS public.marker_images
(
    id BIGSERIAL NOT NULL,
    path text NOT NULL,
    refers_to BIGINT NOT NULL,
    approved BOOLEAN DEFAULT false,
    CONSTRAINT marker_images_id_pkey PRIMARY KEY (id),
    CONSTRAINT marker_images_refers_to_fkey FOREIGN KEY (refers_to)
        REFERENCES public.markers (id) MATCH SIMPLE
        ON UPDATE cascade
        ON DELETE cascade
);
