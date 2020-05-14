using System;
using System.Linq;
using System.Threading.Tasks;
using DSharpPlus.Entities;
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
        private readonly GuildInfoHelper _guildInfoHelper;
        private readonly PermissionsHandler _permissionsHandler;
        public PrefixHelper(RoyalGuardContext context, StringRenderer stringRenderer, TrieHandler trieHandler, GuildInfoHelper guildInfoHelper, PermissionsHandler permissionsHandler)
        {
            _context = context;
            _stringRenderer = stringRenderer;
            _trieHandler = trieHandler;
            _guildInfoHelper = guildInfoHelper;
            _permissionsHandler = permissionsHandler;
        }

        /*
         * Gets the prefix from the trieHandler
         * If the prefix is the same as the default prefix, run AddPrefix
         * Otherwise update the existing prefix in the db and trie
         */
        public async Task HandleConfiguration(DiscordMessage message, bool emergency)
        {
            int messageCountCheck;

            if (emergency)
                messageCountCheck = 3;
            else
                messageCountCheck = 2;

            if (_stringRenderer.GetMessageCount(message) < messageCountCheck)
            {
                await GetPrefix(message);
                return;
            }

            if (!_permissionsHandler.CheckPermission(message, DSharpPlus.Permissions.ManageMessages))
                return;

            if (!await _guildInfoHelper.EnsureGuild(message.Channel.GuildId))
                _guildInfoHelper.AddNewEntry(message.Channel.GuildId);

            string newPrefix = _stringRenderer.GetWordFromIndex(message, messageCountCheck - 1);

            var result = _trieHandler.GetPrefix(message.Channel.GuildId);

            if (result.Equals(CredentialsHelper.DefaultPrefix) || result == null) 
                await SetPrefix(message.Channel.GuildId, newPrefix);
            else
            {
                await UpdatePrefix(message.Channel.GuildId, newPrefix);
                _trieHandler.AddToTrie(message.Channel.GuildId, newPrefix);
            }

            await message.RespondAsync($"My new prefix is `{newPrefix}` for `{message.Channel.Guild.Name}`!");
        }

        // Wrapper for trieHandler prefix fetching
        private async Task GetPrefix(DiscordMessage message)
        {
            var curPrefix = _trieHandler.GetPrefix(message.Channel.GuildId);

            await message.RespondAsync($"My prefix for `{message.Channel.Guild.Name}` is `{curPrefix}`");
        }

        // Updates the prefix in the Database
        private async Task UpdatePrefix(ulong guildId, string newPrefix)
        {
            var result = await _context.GuildInfoStore
                .FirstOrDefaultAsync(q => q.GuildId.Equals(guildId));

            if (newPrefix == CredentialsHelper.DefaultPrefix)
                result.Prefix = null;
            else
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
        private async Task SetPrefix(ulong guildId, string prefix)
        {
            await UpdatePrefix(guildId, prefix);

            _trieHandler.AddToTrie(guildId, prefix);
        }

        public async Task ResetPrefix(DiscordMessage message)
        {
            var result = await _context.GuildInfoStore
                .FirstOrDefaultAsync(q => q.GuildId.Equals(message.Channel.GuildId));

            if (result != null)
            {
                _trieHandler.RemovePrefix(message.Channel.GuildId);
                result.Prefix = null;

                if (result == null)
                    _context.Remove(result);

                await _context.SaveChangesAsync();
            }
            
            await message.RespondAsync($"Reset the prefix back to `{CredentialsHelper.DefaultPrefix}`!");
        }

        // Load all prefixes from the database into the globalTrie
        public async Task LoadPrefix() 
        {
            var result = await _context.GuildInfoStore.ToListAsync();

            foreach (var i in result) 
                _trieHandler.AddToTrie(i.GuildId, i.Prefix);
        }

        // Modular help for prefixes
        public static async Task PrefixHelp(DiscordMessage message)
        {
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();
            eb.WithTitle("Custom Prefix Help");
            eb.WithDescription("Description: Commands for custom bot prefixes");
            eb.AddField("Commands", "prefix <character>: Sets the server's bot prefix to a single character prefix \n\n");

            await message.RespondAsync("", false, eb.Build());
        }
    }
}