CREATE TABLE IF NOT EXISTS public.trash_type
(
    id integer NOT NULL,
    name text COLLATE pg_catalog."default" NOT NULL,
    CONSTRAINT trash_type_pkey PRIMARY KEY (id)
)

TABLESPACE pg_default;

ALTER TABLE IF EXISTS public.trash_type
    OWNER to mindshub;

INSERT INTO public.trash_type (
	id, name)
	VALUES 
  (1, 'unknown'),
  (2, 'plastic'),
  (3, 'paper'),
  (4, 'undifferentiated'),
  (5, 'glass'),
  (6, 'compost'),
  (7, 'electronics');