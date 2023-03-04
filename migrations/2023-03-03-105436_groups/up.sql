-- Table: public.groups

CREATE TABLE IF NOT EXISTS public.groups
(
    id BIGSERIAL NOT NULL,
    name TEXT NOT NULL,
    CONSTRAINT groups_id_pkey PRIMARY KEY (id)
)

TABLESPACE pg_default;

ALTER TABLE IF EXISTS public.groups
    OWNER to mindshub;
