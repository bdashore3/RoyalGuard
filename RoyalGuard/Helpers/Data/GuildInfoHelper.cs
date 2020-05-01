using System;
using System.Threading.Tasks;
using Microsoft.EntityFrameworkCore;

namespace RoyalGuard.Helpers.Data
{
    public class GuildInfoHelper
    {
        private readonly RoyalGuardContext _context;
        public GuildInfoHelper(RoyalGuardContext context)
        {
            _context = context;
        }
        public async Task<bool> EnsureGuild(ulong guildId)
        {
            return await _context.GuildInfoStore.AnyAsync(q => q.GuildId == guildId);
        }
        public async Task AddNewEntry(ulong guildId, string prefix = null, ulong mutedRoleId = 0, ulong muteChannelId = 0)
        {
            GuildInfo FileToAdd = new GuildInfo
            {
                GuildId = guildId,
                Prefix = prefix,
                MutedRoleId = mutedRoleId,
                MuteChannelId = muteChannelId
            };

            await _context.AddAsync(FileToAdd);
            await _context.SaveChangesAsync();
        }
    }
}
