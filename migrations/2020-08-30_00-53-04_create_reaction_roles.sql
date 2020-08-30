-- Add migration script here
CREATE TABLE public.reaction_roles
(
    message_id bigint NOT NULL,
    guild_id bigint NOT NULL,
    emoji character varying(20) NOT NULL,
    role_id bigint NOT NULL,
    animated boolean,
    emoji_name text,
    PRIMARY KEY (message_id, emoji, role_id),
    CONSTRAINT "FK_reaction_roles_guild_info_guild_id" FOREIGN KEY (guild_id)
        REFERENCES public.guild_info (guild_id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE CASCADE
        NOT VALID
);

ALTER TABLE public.reaction_roles
    OWNER to postgres;