-- Your SQL goes here
CREATE TABLE IF NOT EXISTS public.users
(
    id BIGSERIAL NOT NULL,
    name character varying(254) COLLATE pg_catalog."default" NOT NULL UNIQUE,
    email character varying(254) COLLATE pg_catalog."default" NOT NULL UNIQUE,
    password character varying(255) COLLATE pg_catalog."default" NOT NULL,
    is_admin boolean DEFAULT 'false',
    points DOUBLE PRECISION DEFAULT 0.0,
    CONSTRAINT users_pkey PRIMARY KEY (id),
    CONSTRAINT users_name_key UNIQUE (name)
);
ALTER TABLE IF EXISTS public.users OWNER TO mindshub;
