-- Your SQL goes here
CREATE TABLE IF NOT EXISTS public.groups_groups_join
(
    id BIGSERIAL NOT NULL,
    group_parent_id BIGINT NOT NULL,
    group_son_id BIGINT NOT NULL,
    CONSTRAINT groups_groups_join_id_pkey PRIMARY KEY (id),
    CONSTRAINT groups_groups_join_parent_to_fkey FOREIGN KEY (group_parent_id)
        REFERENCES public.groups (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION,
    CONSTRAINT group_groups_join_son_to_fkey FOREIGN KEY (group_son_id)
        REFERENCES public.groups (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION
);
