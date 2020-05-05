using System;
using System.Threading.Tasks;
using RoyalGuard.Helpers.Data;

namespace RoyalGuard.Handlers
{
    public class ServerJoinHandler
    {
        public readonly RoyalGuardContext _context;
        public ServerJoinHandler(RoyalGuardContext context)
        {
            _context = context;
        }
        public async Task HandleJoin(ulong guildId)
        {
            GuildInfo FileToAdd = new GuildInfo()
            {
                GuildId = guildId,
            };

            await _context.AddAsync(FileToAdd);
            await _context.SaveChangesAsync();
        }
    }
}
