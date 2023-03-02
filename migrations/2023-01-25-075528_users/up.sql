-- Your SQL goes here
CREATE TABLE IF NOT EXISTS public.users
(
    id BIGSERIAL NOT NULL,
    email character varying(254) COLLATE pg_catalog."default" NOT NULL,
    password character varying(255) COLLATE pg_catalog."default" NOT NULL,
    is_admin boolean DEFAULT 'false',
    CONSTRAINT users_pkey PRIMARY KEY (id),
    CONSTRAINT users_email_key UNIQUE (email)
)

TABLESPACE pg_default;

ALTER TABLE IF EXISTS public.users
    OWNER to mindshub;