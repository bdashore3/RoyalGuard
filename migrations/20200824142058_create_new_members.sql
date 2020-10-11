-- Add migration script here
CREATE TABLE public.new_members
(
    guild_id bigint NOT NULL,
    channel_id bigint NOT NULL,
    welcome_message text,
    leave_message text,
    PRIMARY KEY (guild_id),
    CONSTRAINT "FK_new_members_guild_info_guild_id" FOREIGN KEY (guild_id)
        REFERENCES public.guild_info (guild_id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE CASCADE
        NOT VALID
);

ALTER TABLE public.new_members
    OWNER to postgres;

CREATE TABLE public.welcome_roles
(
    guild_id bigint NOT NULL,
    role_id bigint NOT NULL,
    PRIMARY KEY (guild_id, role_id),
    CONSTRAINT "FK_welcome_roles_guild_info_guild_id" FOREIGN KEY (guild_id)
        REFERENCES public.guild_info (guild_id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE CASCADE
        NOT VALID
);

ALTER TABLE public.welcome_roles
    OWNER to postgres;
