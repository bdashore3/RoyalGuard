using System;
using System.Threading.Tasks;
using DSharpPlus.Entities;
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
            return await _context.GuildInfoStore.AnyAsync(q => q.GuildId.Equals(guildId));
        }

        public async Task GuildSetup(DiscordMessage message)
        {
            if (await EnsureGuild(message.Channel.GuildId))
            {
                await message.RespondAsync("You have already initialized your guild in the database!");
                return;
            }

            AddNewEntry(message.Channel.GuildId);
            await message.RespondAsync("Sucessfully re-added your guild to the database!");
        }

        public async Task NewGuildAdded(ulong guildId)
        {
            var result = await _context.GuildInfoStore
                .FirstOrDefaultAsync(q => q.GuildId.Equals(guildId));
            
            if (result != null)
            {
                result.DeleteTime = 0;
                await _context.SaveChangesAsync();
            }
            else
                AddNewEntry(guildId);
        }

        public void AddNewEntry(ulong guildId)
        {
            GuildInfo FileToAdd = new GuildInfo
            {
                GuildId = guildId
            };

            _context.Add(FileToAdd);
            _context.SaveChanges();
        }

        public async Task InitialRemoval(ulong guildId)
        {
            var result = await _context.Mutes.ToListAsync();

            foreach(var i in result)
            {
                if (i.GuildId.Equals(guildId))
                {
                    _context.Remove(i);
                }
            }

            await _context.SaveChangesAsync();

            await FlagForRemoval(guildId);
        }

        public async Task FlagForRemoval(ulong guildId)
        {
            var result = await _context.GuildInfoStore
                .FirstOrDefaultAsync(q => q.GuildId.Equals(guildId));
            
            result.DeleteTime = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds() + 30000;

            await _context.SaveChangesAsync();
        }

        public async Task CheckForRemoval()
        {
            long curTime = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();

            var result = await _context.GuildInfoStore.ToListAsync();

            foreach(var i in result)
            {
                if (i.DeleteTime != 0)
                {
                    if (i.DeleteTime <= curTime)
                    {
                        Console.WriteLine("Deleting entry!");
                        _context.Remove(i);
                        await _context.SaveChangesAsync();
                    }
                }
            }
        }
    }
}
