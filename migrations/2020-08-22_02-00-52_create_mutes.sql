-- Add migration script here
ALTER TABLE public.guild_info
    ADD COLUMN muted_role_id bigint;

ALTER TABLE public.guild_info
    ADD COLUMN mute_channel_id bigint;
