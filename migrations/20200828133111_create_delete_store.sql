-- Add migration script here
CREATE TABLE public.delete_time_store
(
    guild_id bigint NOT NULL,
    delete_time bigint NOT NULL,
    PRIMARY KEY (guild_id),
    CONSTRAINT "FK_delete_time_store_guild_info_guild_id" FOREIGN KEY (guild_id)
        REFERENCES public.guild_info (guild_id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE CASCADE
        NOT VALID
);

ALTER TABLE public.delete_time_store
    OWNER to postgres;