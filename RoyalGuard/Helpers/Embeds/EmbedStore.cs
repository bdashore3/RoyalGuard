using System;
using System.Collections.Generic;
using DSharpPlus.Entities;

namespace RoyalGuard.Helpers
{
    public class EmbedStore
    {
        // Stores all embeds for reference later. Considered as extension methods.
        public static DiscordEmbed GetBanEmbed(string avatarUrl, string username, string reason, bool useId)
        {
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();

            eb.WithColor(DiscordColor.Red);

            if (useId)
                eb.WithTitle("New Ban by ID");
            else
                eb.WithTitle("New Ban");

            if(avatarUrl != null)
                eb.WithThumbnailUrl(avatarUrl);

            eb.AddField("Username ", username);
            eb.AddField("Reason", reason);

            return eb.Build();
        }

        public static DiscordEmbed GetUnbanEmbed(string username, bool useId)
        {
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();

            eb.WithColor(DiscordColor.Green);

            if (useId)
                eb.WithTitle("New Unban by ID");
            else
                eb.WithTitle("New Unban");

            eb.WithDescription($"Username: {username}");

            return eb.Build();
        }

        public static DiscordEmbed GetWarnEmbed(string avatarUrl, string username, string warnNumberSend, bool newWarn)
        {
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();

            if (newWarn)
            {
                eb.WithColor(DiscordColor.IndianRed);
                eb.WithTitle("New Warn");
            }
            else
            {
                eb.WithColor(DiscordColor.Green);
                eb.WithTitle("Removed Warn");
            }
            eb.WithThumbnailUrl(avatarUrl);
            eb.AddField("Username", username);
            eb.AddField("Warn Amount",  warnNumberSend);

            return eb.Build();
        }

        public static DiscordEmbed GetMuteEmbed(string avatarUrl, string username, bool newMute, bool usingTime, string muteTimeLength = null)
        {
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();

            eb.WithThumbnailUrl(avatarUrl);
            eb.AddField("Username", username);

            if (newMute)
            {
                eb.WithColor(DiscordColor.IndianRed);
                eb.WithTitle("New Mute");
                if (usingTime)
                {
                    eb.WithDescription("This mute will expire after the given time!");
                    eb.AddField("Time Length", muteTimeLength);
                }
                else
                {
                    eb.WithDescription("This mute has to be removed by an admin!");  
                }
            }
            else
            {
                eb.WithColor(DiscordColor.Green);
                eb.WithTitle("Removed Mute");
            }

            return eb.Build();
        }

        public static DiscordEmbed ChannelEmbed(string channelType, ulong channelId)
        {
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();

            eb.WithColor(DiscordColor.Turquoise);
            eb.WithTitle($"New {channelType} Channel");
            eb.WithDescription($"New Channel: <#{channelId}>");

            return eb.Build();
        }

        public static DiscordEmbed NewMemberInfoEmbed(string type, string message, ulong channelId)
        {
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();

            eb.WithColor(DiscordColor.Turquoise);
            eb.WithTitle($"{type} message");
            eb.WithDescription($"{message} \nCurrent welcome/leave channel: <#{channelId}>");

            return eb.Build();
        }

        public static DiscordEmbed NewMemberRolesEmbed(DiscordGuild guild, List<(ulong Id, bool exists)> roleIds, bool addRole)
        {
            List<string> newRoleList = new List<string>();
            List<string> existingRoleList = new List<string>();
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();

            if (addRole)
            {
                eb.WithTitle("New Welcome Roles");
                eb.WithColor(DiscordColor.Green);
            }
            else
            {
                eb.WithTitle("Removed Welcome Roles");
                eb.WithColor(DiscordColor.IndianRed);
            }

            foreach (var i in roleIds)
            {
                DiscordRole newRole = guild.GetRole(i.Id);

                if (!i.exists)
                    newRoleList.Add(newRole.Mention);
                else
                    existingRoleList.Add(newRole.Mention);
            }

            if (newRoleList.Count == 0)
            {
                if (addRole)
                    newRoleList.Add("No new roles to add.");
                else
                    newRoleList.Add("No new roles to remove");
            }
            else if (existingRoleList.Count == 0)
            {
                if (addRole)
                    existingRoleList.Add("All roles have been added.");
                else
                    existingRoleList.Add("All roles have been removed");
            }

            if (addRole)
            {
                eb.AddField("Roles Added", $"{String.Join(" \n", newRoleList.ToArray())}");
                eb.AddField("Roles that already exist", $"{String.Join(" \n", existingRoleList.ToArray())}");
            }
            else
            {
                eb.AddField("Roles Removed", $"{String.Join(" \n", existingRoleList.ToArray())}");
                eb.AddField("Roles that don't exist", $"{String.Join(" \n", newRoleList.ToArray())}");
            }

            return eb.Build();
        }

        public static DiscordEmbed NewMemberRolesInfo(DiscordGuild guild, List<ulong> roleIds)
        {
            string[] roleMentions = new string[roleIds.Count];
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();

            eb.WithColor(DiscordColor.Turquoise);
            eb.WithTitle("Roles Assigned on Welcome");

            for (int i = 0; i < roleMentions.Length; i++)
            {
                DiscordRole role = guild.GetRole(roleIds[i]);
                roleMentions[i] = role.Mention;
            }

            eb.WithDescription($"{String.Join(" ", roleMentions)}");

            return eb.Build();
        }
    }
}
