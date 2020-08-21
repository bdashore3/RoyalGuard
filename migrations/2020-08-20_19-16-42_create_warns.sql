-- Add migration script here
CREATE TABLE public.warns
(
    user_id bigint NOT NULL,
    guild_id bigint NOT NULL,
    warn_number integer NOT NULL,
    PRIMARY KEY (user_id, guild_id),
    CONSTRAINT "FK_warns_guild_info_guild_id" FOREIGN KEY (guild_id)
        REFERENCES public.guild_info (guild_id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE CASCADE
        NOT VALID
);

ALTER TABLE public.warns
    OWNER to postgres;