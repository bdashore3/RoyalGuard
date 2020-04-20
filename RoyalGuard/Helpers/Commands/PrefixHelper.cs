using System;
using System.Threading.Tasks;
using DSharpPlus.Entities;
using KTrie;
using Microsoft.EntityFrameworkCore;
using RoyalGuard.Helpers.Data;
using RoyalGuard.Helpers.Security;

namespace RoyalGuard.Helpers.Commands
{
    public class PrefixHelper
    {
        private readonly RoyalGuardContext _context;
        private readonly StringRenderer _stringRenderer;
        public StringTrie<string> prefixTrie;
        public PrefixHelper(RoyalGuardContext context, StringRenderer stringRenderer)
        {
            _context = context;
            _stringRenderer = stringRenderer;
            prefixTrie = new StringTrie<string>();
        }
        public string FetchPrefix(ulong guildId)
        {
            string result = "";

            if (!prefixTrie.TryGetValue(guildId.ToString(), out result))
            {
                return CredentialsHelper.DefaultPrefix;
            }

            return result;
        }

        public async Task HandleConfiguration(DiscordMessage message)
        {
            string newPrefix = _stringRenderer.GetWordFromIndex(message, 1);

            var result = FetchPrefix(message.Channel.GuildId);

            if (result == CredentialsHelper.DefaultPrefix)             
                await SetPrefix(message.Channel.GuildId, newPrefix);
            else
            {
                await UpdatePrefix(message.Channel.GuildId, newPrefix);
                prefixTrie.Remove(message.Channel.GuildId.ToString());
                prefixTrie.Add(message.Channel.GuildId.ToString(), newPrefix);
            }

            await message.RespondAsync($"My new prefix is `{newPrefix}` for `{message.Channel.Guild.Name}`!");
        }

        public async Task GetPrefix(DiscordMessage message)
        {
            var curPrefix = FetchPrefix(message.Channel.GuildId);

            await message.RespondAsync($"My prefix for `{message.Channel.Guild.Name}` is `{curPrefix}`");
        }

        public async Task UpdatePrefix(ulong guildId, string newPrefix)
        {
            var result = await _context.CustomPrefixes
                .FirstOrDefaultAsync(q => q.GuildId.Equals(guildId));

            result.Prefix = newPrefix;
            await _context.SaveChangesAsync();
        }

        public async Task SetPrefix(ulong guildId, string prefix)
        {
            CustomPrefix FileToAdd = new CustomPrefix
            {
                GuildId = guildId,
                Prefix = prefix            
            };
            
            await _context.AddAsync(FileToAdd);
            prefixTrie.Remove(guildId.ToString());
            prefixTrie.Add(guildId.ToString(), prefix);
            await _context.SaveChangesAsync();
        }
        public async Task LoadPrefix() 
        {
            var result = await _context.CustomPrefixes.ToListAsync();

            foreach (var i in result) 
            {
                prefixTrie.Add(i.GuildId.ToString(), i.Prefix);
            }
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