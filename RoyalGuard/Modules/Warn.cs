using System;
using System.Linq;
using System.Threading.Tasks;
using DSharpPlus.Entities;
using Microsoft.EntityFrameworkCore;
using RoyalGuard.Helpers.Data;
using RoyalGuard.Helpers.Security;
using RoyalGuard.Handlers;
using RoyalGuard.Helpers;

namespace RoyalGuard.Modules
{
    public class Warns
    {
        private readonly RoyalGuardContext _context;
        private readonly PermissionsHandler _permissions;
        public Warns(RoyalGuardContext context, PermissionsHandler permissions)
        {
            _context = context;
            _permissions = permissions;
        }

        public async Task WarnUser(DiscordMessage message)
        {
            if (message.MentionedUsers.Count < 1)
            {
                await message.RespondAsync("Please mention the user you want to warn!");
                return;
            }

            if (_permissions.CheckAdminFromMention(message.MentionedUsers[0], message.Channel))
            {
                await message.RespondAsync("I can't warn an administrator! Please demote the user and try again.");
                return;
            }

            ulong userId = message.MentionedUsers[0].Id;

            if (message.Author.Id == userId)
            {
                await message.RespondAsync("I don't think you can warn yourself.");
                return;
            }

            ulong guildId = message.Channel.GuildId;
            int warnNumber = await GetWarnNumber(guildId, userId);

            if (warnNumber + 1 == CredentialsHelper.WarnsToBan)
            {
                await message.RespondAsync($"That's `{CredentialsHelper.WarnsToBan}` warns! `{message.MentionedUsers[0].Username}` is banned!");
                DiscordEmbed banEmbed = EmbedStore.GetBanEmbed(message.MentionedUsers[0].AvatarUrl, message.MentionedUsers[0].Username, "Passed the warn limit");
                await message.RespondAsync("", false, banEmbed);
                await message.Channel.Guild.BanMemberAsync(userId, 0, "Passed the warn limit");
                await RemoveEntireWarn(guildId, userId);
                return;
            }
            if (warnNumber == -1)
            {
                await AddWarn(guildId, userId, 1);
                warnNumber = 0;
            }
            else
                await UpdateWarn(guildId, userId, warnNumber + 1);
            

            int warnNumberSend = warnNumber + 1;
            string username = $"<@!{userId}>";

            DiscordEmbed newWarnEmbed = EmbedStore.GetWarnEmbed(message.MentionedUsers[0].AvatarUrl, username, warnNumberSend.ToString(), true);

            await message.RespondAsync("", false, newWarnEmbed);
        }

        public async Task UnwarnUser(DiscordMessage message)
        {
            if (message.MentionedUsers.Count < 1)
            {
                await message.RespondAsync("Please mention the user you want to unwarn!");
                return;
            }

            ulong userId = message.MentionedUsers[0].Id;
            ulong guildId = message.Channel.GuildId;
            int warnNumber = await GetWarnNumber(guildId, userId);

            int warnNumberSend = warnNumber - 1;
            string username = $"<@!{userId}>";

            DiscordEmbed unwarnEmbed = EmbedStore.GetWarnEmbed(message.MentionedUsers[0].AvatarUrl, username, warnNumberSend.ToString(), false);

            if (warnNumber - 1 == 0)
            {
                await message.RespondAsync("", false, unwarnEmbed);
                await RemoveEntireWarn(guildId, userId);
                return;
            }

            if (warnNumber == -1)
            {
                await message.RespondAsync($"`{message.MentionedUsers[0].Username}` has never been warned!");
                return;
            }

            await UpdateWarn(guildId, userId, warnNumber - 1);

            await message.RespondAsync("", false, unwarnEmbed);
        }

        public async Task<int> GetWarnNumber(ulong guildId, ulong userId)
        {
            var result = await _context.Warns
                .Where(q => q.GuildId.Equals(guildId))
                .Where(q => q.UserId.Equals(userId))
                .FirstOrDefaultAsync();

            int warnNumber = result?.WarnNumber ?? -1;
            return warnNumber;
        }
        public async Task AddWarn(ulong guildId, ulong userId, int warnNumber)
        {
            Warn FileToAdd = new Warn
            {
                GuildId = guildId,
                UserId = userId,
                WarnNumber = warnNumber
            };
            
            await _context.AddAsync(FileToAdd);
            await _context.SaveChangesAsync();
        }

        public async Task UpdateWarn(ulong guildId, ulong userId, int warnNumber)
        {
            var result = await _context.Warns
                .Where(q => q.GuildId.Equals(guildId))
                .Where(q => q.UserId.Equals(userId))
                .FirstOrDefaultAsync();

            result.WarnNumber = warnNumber;
            await _context.SaveChangesAsync();
        }

        public async Task RemoveEntireWarn(ulong guildId, ulong userId)
        {
            var key = await _context.Warns
                .Where(q => q.GuildId.Equals(guildId))
                .Where(q => q.UserId.Equals(userId))
                .FirstOrDefaultAsync();

            _context.Remove(key);
            await _context.SaveChangesAsync();
        }

        public async Task GetWarns(DiscordMessage message)
        {
            if (message.MentionedUsers.Count < 1)
            {
                await message.RespondAsync("Please mention the user you want to get the warns for!");
                return;
            }
            int warnNumber = await GetWarnNumber(message.Channel.GuildId, message.MentionedUsers[0].Id);

            if (warnNumber == -1 )
                warnNumber = 0;

            await message.RespondAsync($"{message.MentionedUsers[0].Username} has `{warnNumber}` warns");
        }

        public static async Task WarnHelp(DiscordMessage message)
        {
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();
            eb.WithTitle("Warn Help");
            eb.WithDescription("Description: Commands for warning in a server");
            eb.AddField("Commands", "warn <mention>: Adds a warn to the mentioned user \n\n" +
                                    "unwarn <mention>: Removes a warn from the mentioned user \n\n" +
                                    "getwarn <mention>, Gets the amount of warns for the mentioned user");

            await message.RespondAsync("", false, eb.Build());
        }
    }
}
