using System;
using System.Threading.Tasks;
using DSharpPlus.Entities;
using Microsoft.EntityFrameworkCore;
using RoyalGuard.Helpers.Data;
using RoyalGuard.Helpers.Security;

namespace RoyalGuard.Modules
{
    public class Warns
    {
        private readonly RoyalGuardContext _context;
        public Warns(RoyalGuardContext context)
        {
            _context = context;
        }

        public async Task WarnUser(DiscordMessage message)
        {
            if (message.MentionedUsers.Count < 1)
            {
                await message.RespondAsync("Please mention the user you want to warn!");
                return;
            }

            ulong userId = message.MentionedUsers[0].Id;
            int warnNumber = await GetWarnNumber(userId);

            if (warnNumber + 1 == CredentialsHelper.WarnsToBan)
            {
                await message.RespondAsync($"That's `{CredentialsHelper.WarnsToBan}` warns! `{message.MentionedUsers[0].Username}` is banned!");
                await message.Channel.Guild.BanMemberAsync(userId, 0, "Passed the warn limit");
                await RemoveEntireWarn(userId);
                return;
            }
            if (warnNumber == -1)
            {
                await AddWarn(userId, 1);
                warnNumber = 0;
            }
            else
                await UpdateWarn(userId, warnNumber + 1);
            await message.RespondAsync($"`{message.MentionedUsers[0].Username}` has been warned! Number of warns: `{warnNumber + 1}`.");
        }

        public async Task UnwarnUser(DiscordMessage message)
        {
            if (message.MentionedUsers.Count < 1)
            {
                await message.RespondAsync("Please mention the user you want to unwarn!");
                return;
            }

            ulong userId = message.MentionedUsers[0].Id;
            int warnNumber = await GetWarnNumber(userId);

            if (warnNumber - 1 == 0)
            {
                await RemoveEntireWarn(userId);
                await message.RespondAsync($"There are no more warns for `{message.MentionedUsers[0].Username}`");
                return;
            }

            if (warnNumber == -1)
                await message.RespondAsync($"`{message.MentionedUsers[0].Username}` has never been warned!");

            await UpdateWarn(userId, warnNumber - 1);

            await message.RespondAsync($"Removed `1` warn from `{message.MentionedUsers[0].Username}`.");
        }

        public async Task<int> GetWarnNumber(ulong userId)
        {
            var result = await _context.Warns
                .FirstOrDefaultAsync(q => q.DiscordId.Equals(userId));

            int warnNumber = result?.WarnNumber ?? -1;
            return warnNumber;
        }
        public async Task AddWarn(ulong discordId, int warnNumber)
        {
            Warn FileToAdd = new Warn
            {
                DiscordId = discordId,
                WarnNumber = warnNumber
            };
            
            await _context.AddAsync(FileToAdd);
            await _context.SaveChangesAsync();
        }

        public async Task UpdateWarn(ulong discordId, int warnNumber)
        {
            var result = await _context.Warns
                .FirstOrDefaultAsync(q => q.DiscordId.Equals(discordId));
            result.WarnNumber = warnNumber;
            await _context.SaveChangesAsync();
        }

        public async Task RemoveEntireWarn(ulong userId)
        {
            var key = await _context.Warns
                .FirstOrDefaultAsync(q => q.DiscordId == userId);
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
            int warnNumber = await GetWarnNumber(message.MentionedUsers[0].Id);

            if (warnNumber == -1 )
                warnNumber = 0;

            await message.RespondAsync($"`{message.MentionedUsers[0].Username}'s` warn count is `{warnNumber}`");
        }
    }
}
