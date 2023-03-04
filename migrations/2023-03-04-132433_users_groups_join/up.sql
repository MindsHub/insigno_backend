-- Table: public.groups

CREATE TABLE IF NOT EXISTS public.user_groups_join
(
    id BIGSERIAL NOT NULL,
    user_id BIGINT NOT NULL,
    group_id BIGINT NOT NULL,
    CONSTRAINT user_groups_join_id_pkey PRIMARY KEY (id),
    CONSTRAINT user_groups_join_refers_to_fkey FOREIGN KEY (user_id)
        REFERENCES public.users (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION,
    CONSTRAINT user_groups_join_appartains_to_fkey FOREIGN KEY (group_id)
        REFERENCES public.groups (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION
)

TABLESPACE pg_default;

ALTER TABLE IF EXISTS public.groups
    OWNER to mindshub;
