CREATE TABLE IF NOT EXISTS public.marker_types
(
    id BIGSERIAL NOT NULL,
    name TEXT COLLATE pg_catalog."default" NOT NULL,
    points DOUBLE PRECISION NOT NULL,
    CONSTRAINT markers_types_pkey PRIMARY KEY (id)
);

INSERT INTO public.marker_types (
	id, name, points)
	VALUES
  (1, 'unknown',    10.0),
  (2, 'plastic',    10.0),
  (3, 'paper',      10.0),
  (4, 'undifferentiated', 10.0),
  (5, 'glass',      10.0),
  (6, 'compost',    10.0),
  (7, 'electronics', 10.0);
