using System;
using System.Threading.Tasks;
using DSharpPlus.Entities;
using Microsoft.EntityFrameworkCore;
using RoyalGuard.Helpers;
using RoyalGuard.Helpers.Commands;
using RoyalGuard.Helpers.Data;
using RoyalGuard.Helpers.Security;

namespace RoyalGuard.Handlers
{
    public class NewMemberHandler
    {
        private readonly RoyalGuardContext _context;
        private readonly StringRenderer _stringRenderer;
        private readonly TrieHandler _trieHandler;
        public NewMemberHandler(RoyalGuardContext context, StringRenderer stringRenderer, TrieHandler trieHandler)
        {
            _context = context;
            _stringRenderer = stringRenderer;
            _trieHandler = trieHandler;
        }

        public async Task OnNewMemberEvent(DiscordGuild guild, DiscordMember memberObject, string parameter)
        {
            string message = null;
            string member = $"<@!{memberObject.Id}>";

            var result = await _context.NewMembers
                .FirstOrDefaultAsync(q => q.GuildId.Equals(guild.Id));

            switch(parameter)
            {
                case "leave":

                    // Don't send the message if it doesn't exist
                    if (result.LeaveMessage == null)
                        return; 

                    // Replace any custom variables with their local counterparts
                    message = result.LeaveMessage
                        .Replace("{member}", member)
                        .Replace("{user}", member)
                        .Replace("{server}", guild.Name);

                    break;
                
                case "welcome":
                    if (result.WelcomeMessage == null)
                        return;

                    // Replace any custom variables with their local counterparts
                    message = result.WelcomeMessage
                        .Replace("{member}", member)
                        .Replace("{user}", member)
                        .Replace("{server}", guild.Name);
                    
                    break;
            }

            DiscordChannel channel = guild.GetChannel(result.ChannelId);
            string server = guild.Name;
            await channel.SendMessageAsync(message);
        }

        public async Task HandleConfiguration(DiscordMessage message, string parameter)
        {
            string instruction = _stringRenderer.GetWordFromIndex(message, 1);
            string prefix = _trieHandler.GetPrefix(message.Channel.GuildId);

            switch(instruction)
            {   
                case "channel":
                    await SetChannel(message.Channel.GuildId, message.MentionedChannels[0].Id);
                    await message.RespondAsync("", false, EmbedStore.ChannelEmbed("New Member", message.MentionedChannels[0].Id));
                    break;
                
                case "set":
                    string newMessage = _stringRenderer.RemoveExtras(message, 2);
                    await SetMessage(message.Channel.GuildId, message.Channel.Id, newMessage, parameter);
                    await message.RespondAsync($"`{parameter}` message sucessfully set!");
                    break;
                
                case "get":
                    await GetMessage(message.Channel.GuildId, message.Channel, parameter);
                    break;
                
                case "clear":
                    bool finishClear = await ClearMessage(message.Channel.GuildId, parameter);
                    if (finishClear)
                        await message.RespondAsync($"`{parameter}` message sucessfully cleared!");
                    else
                        await message.RespondAsync($"`{parameter}` message doesn't exist! Did you not set it?");
                    break;
                
                case "clearall":
                case "purge":
                    await ClearMessage(message.Channel.GuildId, "all");
                    await message.RespondAsync($"You have been wiped from the database. \nPlease run the welcome set or leave set command if you want to re-add the messages");
                    break;
            }
        }

        // Required to register the WelcomeMessage in the database.

        // TODO: Make this automatic
        public async Task InitialSetup(ulong guildId, ulong channelId, string welcomeMessage = null, string leaveMessage = null)
        {
            NewMember FileToAdd = new NewMember
            {
                GuildId = guildId,
                ChannelId = channelId,
                WelcomeMessage = welcomeMessage,
                LeaveMessage = leaveMessage
            };

            await _context.AddAsync(FileToAdd);
            await _context.SaveChangesAsync();
        }

        /*
         * Set the Welcome/leave channel
         * The default channel is where the init command is ran
         */
        public async Task SetChannel(ulong guildId, ulong channelId)
        {
            var result = await _context.NewMembers
                .FirstOrDefaultAsync(q => q.GuildId.Equals(guildId));

            result.ChannelId = channelId;
            await _context.SaveChangesAsync();
        }

        /*
         * Set the welcome/leave message
         * Welcome/leave toggle is decided by the parameter argument
         */
        public async Task SetMessage(ulong guildId, ulong channelId, string newMessage, string parameter)
        {
            var result = await _context.NewMembers
                .FirstOrDefaultAsync(q => q.GuildId.Equals(guildId));

            switch (parameter)
            {
                case "leave":
                    if (result == null)
                        await InitialSetup(guildId, channelId, null, newMessage);
                    else
                        result.LeaveMessage = newMessage;
                    break;

                case "welcome":
                    if (result == null)
                        await InitialSetup(guildId, channelId, newMessage);
                    else
                        result.WelcomeMessage = newMessage;
                    break;
            }

            await _context.SaveChangesAsync();
        }

        /*
         * Clears the welcome message, leave message, or both
         * Case All purges the guild from the database.
         */
        public async Task<bool> ClearMessage(ulong guildId, string parameter)
        {
            var result = await _context.NewMembers
                .FirstOrDefaultAsync(q => q.GuildId.Equals(guildId));
            
            if (result == null)
                return false;

            switch (parameter)
            {
                case "leave":
                    if (result.LeaveMessage == null)
                        return false;

                    result.LeaveMessage = null;
                    break;
                
                case "welcome":
                    if (result.WelcomeMessage == null)
                        return false;

                    result.WelcomeMessage = null;
                    break;
                
                case "all":
                    _context.Remove(result);
                    break;
            }

            await _context.SaveChangesAsync();
            return true;
        }

        /*
         * Get the Welcome/Leave message
         *
         * Possible TODO: Make this use a trie
         */
        public async Task GetMessage(ulong guildId, DiscordChannel channel, string parameter)
        {
            var result = await _context.NewMembers
                .FirstOrDefaultAsync(q => q.GuildId.Equals(guildId));

            switch (parameter)
            {
                case "leave":
                    if (result.LeaveMessage == null)
                        result.LeaveMessage = "No message set.";

                    await channel.SendMessageAsync("", false, EmbedStore.NewMemberInfoEmbed(parameter, result.LeaveMessage, result.ChannelId));
                    break;
                
                case "welcome":
                    if (result.WelcomeMessage == null)
                        result.WelcomeMessage = "No message set.";
               
                    await channel.SendMessageAsync("", false, EmbedStore.NewMemberInfoEmbed(parameter, result.WelcomeMessage, result.ChannelId));
                    break;
            }
        }

        // Modular help command: New member section. Referenced in help welcome or help leave
        public static async Task NewMemberHelp(DiscordMessage message)
        {
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();

            eb.WithTitle("Welcome/Leave message help");
            eb.WithDescription("Setting server welcome/leave messages");
            eb.AddField("Commands", "welcome <subcommand>: Used for welcome messages \n\n" +
                                    "leave <subcommand>: Used for leave messages");
            eb.AddField("SubCommands (Can be used with welcome or leave commands)", "set <new message>: Sets the welcome/leave message. You can use {user} or {member} to specify the joined user and {server} to specify the server name \n\n" +
                                    "channel <channel Id>: Sets the channel where the messages are sent. Default channel is where you inited. \n\n" +
                                    "get: Gets the welcome/leave message \n\n" +
                                    "clear: Removes the current welcome OR leave message. If you don't want to use RoyalGuard for welcome/leave messages, use purge or clearall! \n\n" +
                                    "purge: Removes the welcome/leave database entry. ONLY use this if you don't want to use RoyalGuard for welcomes/leaves!");
            
            await message.RespondAsync("", false, eb.Build());
        }
    }
}
