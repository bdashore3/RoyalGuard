using System;
using System.Linq;
using System.Threading.Tasks;
using DSharpPlus.Entities;
using Microsoft.EntityFrameworkCore;
using RoyalGuard.Helpers.Data;
using RoyalGuard.Helpers.Security;
using RoyalGuard.Handlers;
using RoyalGuard.Helpers;
using RoyalGuard.Helpers.Commands;

namespace RoyalGuard.Modules
{
    public class Warns
    {
        private readonly RoyalGuardContext _context;
        private readonly PermissionsHandler _permissions;
        private readonly TrieHandler _trieHandler;
        private readonly StringRenderer _stringRenderer;
        public Warns(RoyalGuardContext context, PermissionsHandler permissions, TrieHandler trieHandler, StringRenderer stringRenderer)
        {
            _context = context;
            _permissions = permissions;
            _trieHandler = trieHandler;
            _stringRenderer = stringRenderer;
        }

        public async Task WarnUser(DiscordMessage message)
        {
            if (_stringRenderer.GetMessageCount(message) < 2)
            {
                await WarnHelp(message);
                return;
            }

            // If there's no mention
            if (message.MentionedUsers.Count == 0)
            {
                await message.RespondAsync("Please mention the user you want to warn!");
                return;
            }

            // Check if the user is warning an admin
            if (_permissions.CheckAdminFromMention(message.MentionedUsers[0], message.Channel))
            {
                await message.RespondAsync("I can't warn an administrator! Please demote the user and try again.");
                return;
            }

            ulong userId = message.MentionedUsers[0].Id;

            // Check if you're warning yourself
            if (message.Author.Id == userId)
            {
                await message.RespondAsync("I don't think you can warn yourself.");
                return;
            }

            ulong guildId = message.Channel.GuildId;
            int warnNumber = await GetWarnNumber(guildId, userId);

            // If the warn number hits the ban limit, ban the user with a reason

            // TODO: Set WarnsToBan through the DB for every server
            if (warnNumber + 1 == 3)
            {
                await message.RespondAsync($"That's 3 warns! `{message.MentionedUsers[0].Username}` is banned!");
                DiscordEmbed banEmbed = EmbedStore.GetBanEmbed(message.MentionedUsers[0].AvatarUrl, message.MentionedUsers[0].Username, "Passed the warn limit");
                await message.RespondAsync("", false, banEmbed);
                await message.Channel.Guild.BanMemberAsync(userId, 0, "Passed the warn limit");
                await RemoveEntireWarn(guildId, userId);
                return;
            }

            // If there are no warns, add a new Database Entry
            if (warnNumber == -1)
            {
                await AddWarn(guildId, userId, 1);
                warnNumber = 0;
            }

            // Update the existing warn number
            else
                await UpdateWarn(guildId, userId, warnNumber + 1);
            

            // Send the warn number + 1 since we're adding one warn
            int warnNumberSend = warnNumber + 1;
            string username = $"<@!{userId}>";

            DiscordEmbed newWarnEmbed = EmbedStore.GetWarnEmbed(message.MentionedUsers[0].AvatarUrl, username, warnNumberSend.ToString(), true);

            await message.RespondAsync("", false, newWarnEmbed);
        }

        public async Task UnwarnUser(DiscordMessage message)
        {
            if (_stringRenderer.GetMessageCount(message) < 2)
            {
                await WarnHelp(message);
                return;
            }

            // If there's no mention
            if (message.MentionedUsers.Count == 0)
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

            // If the warns after removal equal 0, remove the database entry
            if (warnNumber - 1 == 0)
            {
                await message.RespondAsync("", false, unwarnEmbed);
                await RemoveEntireWarn(guildId, userId);
                return;
            }

            // If there are no warns, say so
            if (warnNumber == -1)
            {
                await message.RespondAsync($"`{message.MentionedUsers[0].Username}` has never been warned!");
                return;
            }

            // Update the warn count
            await UpdateWarn(guildId, userId, warnNumber - 1);

            await message.RespondAsync("", false, unwarnEmbed);
        }

        public async Task<int> GetWarnNumber(ulong guildId, ulong userId)
        {
            var result = await _context.Warns
                .Where(q => q.GuildId.Equals(guildId))
                .Where(q => q.UserId.Equals(userId))
                .FirstOrDefaultAsync();

            // Return the warnNumber if not null. Otherwise return -1
            int warnNumber = result?.WarnNumber ?? -1;
            return warnNumber;
        }

        // Add a new warn
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

        // Update an existing warn
        public async Task UpdateWarn(ulong guildId, ulong userId, int warnNumber)
        {
            var result = await _context.Warns
                .Where(q => q.GuildId.Equals(guildId))
                .Where(q => q.UserId.Equals(userId))
                .FirstOrDefaultAsync();

            result.WarnNumber = warnNumber;
            await _context.SaveChangesAsync();
        }

        // Get rid of the database entry
        public async Task RemoveEntireWarn(ulong guildId, ulong userId)
        {
            var key = await _context.Warns
                .Where(q => q.GuildId.Equals(guildId))
                .Where(q => q.UserId.Equals(userId))
                .FirstOrDefaultAsync();

            _context.Remove(key);
            await _context.SaveChangesAsync();
        }

        // Gets the warns from the user. If the user doesn't have warns, respond with 0
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

        // Modular help function
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
