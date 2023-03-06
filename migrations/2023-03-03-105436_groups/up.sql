-- Table: public.groups

CREATE TABLE IF NOT EXISTS public.groups
(
    id BIGSERIAL NOT NULL,
    name TEXT NOT NULL,
    points DOUBLE PRECISION DEFAULT 0.0,
    creation_date timestamp with time zone DEFAULT NOW(),
    end_date timestamp with time zone,
    CONSTRAINT groups_id_pkey PRIMARY KEY (id)
)

TABLESPACE pg_default;

ALTER TABLE IF EXISTS public.groups
    OWNER to mindshub;
