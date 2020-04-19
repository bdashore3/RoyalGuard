using System;
using System.Threading.Tasks;
using DSharpPlus.Entities;
using Microsoft.EntityFrameworkCore;
using RoyalGuard.Helpers.Data;
using RoyalGuard.Helpers.Security;

namespace RoyalGuard.Helpers.Commands
{
    public class PrefixHelper
    {
        private readonly RoyalGuardContext _context;
        private readonly StringRenderer _stringRenderer;
        public PrefixHelper(RoyalGuardContext context, StringRenderer stringRenderer)
        {
            _context = context;
            _stringRenderer = stringRenderer;
        }
        public async Task<string> FetchPrefix(ulong guildId)
        {
            var result = await _context.CustomPrefixes
                .FirstOrDefaultAsync(q => q.GuildId.Equals(guildId));
            
            if (result == null)
                return CredentialsHelper.DefaultPrefix;
            
            return result.Prefix;
        }

        public async Task HandleConfiguration(DiscordMessage message)
        {
            string newPrefix = _stringRenderer.GetWordFromIndex(message, 1);

            var result = await _context.CustomPrefixes
                .FirstOrDefaultAsync(q => q.GuildId.Equals(message.Channel.GuildId));

            if (result == null)             
                await SetPrefix(message.Channel.GuildId, newPrefix);
            else 
            {
                result.Prefix = newPrefix;
                await _context.SaveChangesAsync();
            }

            await message.RespondAsync($"My new prefix is `{newPrefix}` for `{message.Channel.Guild.Name}`!");
        }

        public async Task GetPrefix(DiscordMessage message)
        {
            var curPrefix = await FetchPrefix(message.Channel.GuildId);

            await message.RespondAsync($"My prefix for `{message.Channel.Guild.Name}` is `{curPrefix}`");
        }

        public async Task SetPrefix(ulong guildId, string prefix)
        {
            CustomPrefix FileToAdd = new CustomPrefix
            {
                GuildId = guildId,
                Prefix = prefix            
            };
            
            await _context.AddAsync(FileToAdd);
            await _context.SaveChangesAsync();
        }

        public static async Task PrefixHelp(DiscordMessage message)
        {
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();
            eb.WithTitle("Custom Prefix Help");
            eb.WithDescription("Description: Commands for custom bot prefixes");
            eb.AddField("Commands", "prefix <character>: Sets the server's bot prefix to a single character prefix \n\n" +
                                    "getprefix: Get's the server's current command prefix \n\n");

            await message.RespondAsync("", false, eb.Build());
        }
    }
}