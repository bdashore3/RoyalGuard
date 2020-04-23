using System;
using System.Threading.Tasks;
using DSharpPlus.Entities;
using KTrie;
using Microsoft.EntityFrameworkCore;
using RoyalGuard.Handlers;
using RoyalGuard.Helpers.Data;
using RoyalGuard.Helpers.Security;

namespace RoyalGuard.Helpers.Commands
{
    public class PrefixHelper
    {
        // Variables and constructor for DI
        private readonly RoyalGuardContext _context;
        private readonly StringRenderer _stringRenderer;
        private readonly TrieHandler _trieHandler;
        public PrefixHelper(RoyalGuardContext context, StringRenderer stringRenderer, TrieHandler trieHandler)
        {
            _context = context;
            _stringRenderer = stringRenderer;
            _trieHandler = trieHandler;
        }

        /*
         * Gets the prefix from the trieHandler
         * If the prefix is the same as the default prefix, run AddPrefix
         * Otherwise update the existing prefix in the db and trie
         */
        public async Task HandleConfiguration(DiscordMessage message)
        {
            string newPrefix = _stringRenderer.GetWordFromIndex(message, 1);

            var result = _trieHandler.GetPrefix(message.Channel.GuildId);

            if (result == CredentialsHelper.DefaultPrefix) 
                await SetPrefix(message.Channel.GuildId, newPrefix);
            else
            {
                await UpdatePrefix(message.Channel.GuildId, newPrefix);
                _trieHandler.AddToTrie(message.Channel.GuildId, newPrefix);
            }

            await message.RespondAsync($"My new prefix is `{newPrefix}` for `{message.Channel.Guild.Name}`!");
        }

        // Wrapper for trieHandler prefix fetching
        public async Task GetPrefix(DiscordMessage message)
        {
            var curPrefix = _trieHandler.GetPrefix(message.Channel.GuildId);

            await message.RespondAsync($"My prefix for `{message.Channel.Guild.Name}` is `{curPrefix}`");
        }

        // Used for checking if the prefix REALLY doesn't exist in the SetPrefix method
        public async Task<bool> GetPrefixFromDatabase(ulong guildId)
        {
            var result = await _context.CustomPrefixes
                .FirstOrDefaultAsync(q => q.GuildId.Equals(guildId));
            
            if (result == null)
                return false;
            
            return true;
        }

        // Updates the prefix in the Database
        public async Task UpdatePrefix(ulong guildId, string newPrefix)
        {
            var result = await _context.CustomPrefixes
                .FirstOrDefaultAsync(q => q.GuildId.Equals(guildId));

            result.Prefix = newPrefix;
            await _context.SaveChangesAsync();
        }

        /*
         * Sets a new guild prefix
         * 
         * If the prefix exists in the database, don't execute a database operation
         * This is because we check if the message has the default prefix, but
         * the user may be using the default prefix to set the server's prefix.
         * 
         * Add the new prefix to the trie regardless.
         */
        public async Task SetPrefix(ulong guildId, string prefix)
        {
            if (!(await GetPrefixFromDatabase(guildId)))
            {
                CustomPrefix FileToAdd = new CustomPrefix
                {
                    GuildId = guildId,
                    Prefix = prefix            
                };
            
                await _context.AddAsync(FileToAdd);
                await _context.SaveChangesAsync();
            }
            else
                await UpdatePrefix(guildId, prefix);

            _trieHandler.AddToTrie(guildId, prefix);
        }

        // Load all prefixes from the database into the globalTrie
        public async Task LoadPrefix() 
        {
            var result = await _context.CustomPrefixes.ToListAsync();

            foreach (var i in result) 
            {
                _trieHandler.AddToTrie(i.GuildId, i.Prefix);
            }
        }

        // Modular help for prefixes
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