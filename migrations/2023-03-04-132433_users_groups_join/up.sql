-- Table: public.groups

CREATE TABLE IF NOT EXISTS public.users_groups_join
(
    id BIGSERIAL NOT NULL,
    user_id BIGINT NOT NULL,
    group_id BIGINT NOT NULL,
    CONSTRAINT users_groups_join_id_pkey PRIMARY KEY (id),
    CONSTRAINT users_groups_join_refers_to_fkey FOREIGN KEY (user_id)
        REFERENCES public.users (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION,
    CONSTRAINT users_groups_join_appartains_to_fkey FOREIGN KEY (group_id)
        REFERENCES public.groups (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION
)
ALTER TABLE IF EXISTS public.users_groups_join OWNER TO mindshub;
