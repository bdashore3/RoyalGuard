using System;
using System.Threading.Tasks;
using DSharpPlus.Entities;
using RoyalGuard.Helpers.Commands;
using RoyalGuard.Handlers;
using RoyalGuard.Helpers;

namespace RoyalGuard.Modules
{
    public class Bans
    {
        // Variables and constructor for DI
        private readonly StringRenderer _stringRenderer;
        private readonly PermissionsHandler _permissions;

        public Bans(StringRenderer stringRenderer, PermissionsHandler permissions)
        {
            _stringRenderer = stringRenderer;
            _permissions = permissions;
        }
        public async Task BanUser(DiscordMessage message)
        {
            ulong userId;
            string avatarUrl = null;

            if (_stringRenderer.GetMessageCount(message) < 2)
            {
                await BanHelp(message);
                return;
            }

            bool useId = TestForId(_stringRenderer.GetWordFromIndex(message, 1));
            
            if (useId)
                userId = UInt64.Parse(_stringRenderer.GetWordFromIndex(message, 1));
            else
            {
                // If there's no mention
                if (message.MentionedUsers.Count == 0)
                {
                    await message.RespondAsync("Please mention the user you want to ban!");
                    return;
                }

                // Make sure the mentioned user isn't an admin
                if (_permissions.CheckAdminFromMention(message.MentionedUsers[0], message.Channel))
                {
                    await message.RespondAsync("I can't ban an administrator! Please demote the user then try again.");
                    return;
                }

                userId = message.MentionedUsers[0].Id;
                avatarUrl = message.MentionedUsers[0].AvatarUrl;
            }

            // Remove all extras to create a reason
            string reason = _stringRenderer.RemoveExtras(message, 2);
            string username = $"<@!{userId}>";
            //await message.Channel.Guild.BanMemberAsync(userId, 0, reason);

            // If there's no reason provided, give something to the embed
            if (reason == null)
                reason = "No reason given.";

            DiscordEmbed banEmbed = EmbedStore.GetBanEmbed(avatarUrl, username, reason, useId);

            await message.RespondAsync("", false, banEmbed);
        }

        public async Task UnbanUser(DiscordMessage message)
        {
            ulong userId;
        
            if (_stringRenderer.GetMessageCount(message) < 2)
            {
                await BanHelp(message);
                return;
            }

            bool useId = TestForId(_stringRenderer.GetWordFromIndex(message, 1));

            // Checks if we're using an ID instead of a mention
            if (useId)
                userId = UInt64.Parse(_stringRenderer.GetWordFromIndex(message, 1));
            else
            {
                // If there's no mention
                if (message.MentionedUsers.Count == 0)
                {
                    await message.RespondAsync("Please mention the user you want to ban!");
                    return;
                }

                userId = message.MentionedUsers[0].Id;
            }

            string username = $"<@!{userId}>";
            await message.Channel.Guild.UnbanMemberAsync(userId);

            DiscordEmbed unbanEmbed = EmbedStore.GetUnbanEmbed(username, useId);
            await message.RespondAsync("", false, unbanEmbed);
        }

        private bool TestForId(string testString)
        {
            try
            {
                UInt64.Parse(testString);
                return true;
            }
            catch (System.FormatException)
            {
                return false;
            }
            catch (System.OverflowException)
            {
                return false;
            }
        }

        public static async Task BanHelp(DiscordMessage message)
        {
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();
            eb.WithTitle("Ban Help");
            eb.WithDescription("Description: Commands for Banning/Unbanning in a server");
            eb.AddField("Commands", "ban <mention> <reason>: Bans a user with a reason \n\n" +
                                    "unban <mention or id>: Unbans the mentioned user or provided ID");

            await message.RespondAsync("", false, eb.Build());
        }
    }
}