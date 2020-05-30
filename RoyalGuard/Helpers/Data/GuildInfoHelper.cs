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

        // Make sure the guild exists
        public async Task<bool> EnsureGuild(ulong guildId)
        {
            return await _context.GuildInfoStore.AnyAsync(q => q.GuildId.Equals(guildId));
        }

        // Setup the guild by the emergency init command 
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

        // When the bot joins a new guild, do this
        public async Task NewGuildAdded(ulong guildId)
        {
            var result = await _context.DeleteTimeStore
                .FirstOrDefaultAsync(q => q.GuildInfoGuildId.Equals(guildId));
            
            if (result != null)
            {
                _context.Remove(result);
                await _context.SaveChangesAsync();
            }
            else
                AddNewEntry(guildId);
        }

        // Add a new guild entry
        public void AddNewEntry(ulong guildId)
        {
            GuildInfo FileToAdd = new GuildInfo
            {
                GuildId = guildId
            };

            _context.Add(FileToAdd);
            _context.SaveChanges();
        }

        // When the bot is kicked, do these immediately
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

        // Flag the guild ID for server removal within the database
        public async Task FlagForRemoval(ulong guildId)
        {
            var result = await _context.DeleteTimeStore
                .FirstOrDefaultAsync(q => q.GuildInfoGuildId.Equals(guildId));
            
            DeleteTimeInfo FileToAdd = new DeleteTimeInfo
            {
                DeleteTime = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds() + 604800000,
                GuildInfoGuildId = guildId
            };

            await _context.AddAsync(FileToAdd);
            await _context.SaveChangesAsync();
        }

        /*
         * Called by GuildDeleteService
         * Iterates through the delete time list pulled from the database
         * If the delete time is <= the current time, delete the GuildInfo entry
         * This cascades down to all dependents
         */
        public async Task CheckForRemoval()
        {
            long curTime = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();

            var result = await _context.DeleteTimeStore.ToListAsync();

            foreach(var i in result)
            {
                Console.WriteLine($"Checking delete status on guild: {i.GuildInfoGuildId}");

                if (i.DeleteTime <= curTime)
                {
                    Console.WriteLine("Deleting entry!");

                    var guildResult = await _context.GuildInfoStore
                        .FirstOrDefaultAsync(q => q.GuildId.Equals(i.GuildInfoGuildId));
                    
                    _context.Remove(guildResult);

                    await _context.SaveChangesAsync();
                }
                else
                    Console.WriteLine("Entry's time isn't greater than a week! Not deleting!");
            }
        }
    }
}
