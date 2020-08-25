-- Add migration script here
CREATE TABLE public.mutes
(
    guild_id bigint NOT NULL,
    user_id bigint NOT NULL,
    mute_time bigint NOT NULL,
    PRIMARY KEY (guild_id, user_id),
    CONSTRAINT "FK_mutes_guild_info_guild_id" FOREIGN KEY (guild_id)
        REFERENCES public.guild_info (guild_id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE CASCADE
        NOT VALID
);

ALTER TABLE public.mutes
    OWNER to postgres;
