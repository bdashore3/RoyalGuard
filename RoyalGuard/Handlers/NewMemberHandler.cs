using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using DSharpPlus.Entities;
using Microsoft.EntityFrameworkCore;
using RoyalGuard.Helpers;
using RoyalGuard.Helpers.Commands;
using RoyalGuard.Helpers.Data;

namespace RoyalGuard.Handlers
{
    public class NewMemberHandler
    {
        private readonly RoyalGuardContext _context;
        private readonly StringRenderer _stringRenderer;
        private readonly TrieHandler _trieHandler;
        private readonly GuildInfoHelper _guildInfoHelper;
        public NewMemberHandler(RoyalGuardContext context, StringRenderer stringRenderer, TrieHandler trieHandler, GuildInfoHelper guildInfoHelper)
        {
            _context = context;
            _stringRenderer = stringRenderer;
            _trieHandler = trieHandler;
            _guildInfoHelper = guildInfoHelper;
        }

        public async Task OnNewMemberEvent(DiscordGuild guild, DiscordMember memberObject, string parameter)
        {
            string message = null;
            string member = $"<@!{memberObject.Id}>";

            var result = await _context.NewMembers
                .FirstOrDefaultAsync(q => q.GuildInfoGuildId.Equals(guild.Id));

            switch(parameter)
            {
                case "leave":
                    // If the database entry doesn't exist, bail
                    if (result == null)
                        return;

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
                    // If there are welcome roles stored, assign them
                    if (await _context.WelcomeRoles.AnyAsync(q => q.GuildInfoGuildId.Equals(guild.Id)))
                        await AssignRoles(guild, memberObject);

                    // If the database entry doesn't exist, bail
                    if (result == null)
                        return;

                    // Don't send the message if it doesn't exist
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
            int wordCount = _stringRenderer.GetMessageCount(message);

            if (wordCount < 2)
            {
                await NewMemberHelp(message);
                return;
            }

            string instruction = _stringRenderer.GetWordFromIndex(message, 1);

            switch(instruction)
            {   
                // Set the welcome/leave message channel
                case "channel":
                    if (!await SetChannel(message.Channel.GuildId, message.MentionedChannels[0].Id))
                    {
                        await message.RespondAsync($"{parameter} channel isn't set! Please set a welcome message first!");
                        return;
                    }

                    await message.RespondAsync("", false, EmbedStore.ChannelEmbed("Welcome/Leave", message.MentionedChannels[0].Id));
                    break;
                
                // Roles subcommand
                case "roles":
                    if (wordCount < 3)
                    {
                        await NewMemberRolesHelp(message);
                        return;
                    }

                    if (parameter == "welcome")
                        await HandleRoleConfiguration(message, _stringRenderer.GetWordFromIndex(message, 2));
                    else
                        await message.RespondAsync("Please use the `welcome` command to set welcome roles!");
                    break;
                
                // Set the welcome/leave message
                case "set":
                    string newMessage = _stringRenderer.RemoveExtras(message, 2);
                    await SetMessage(message.Channel.GuildId, message.Channel.Id, newMessage, parameter);
                    
                    await message.RespondAsync($"`{parameter}` message sucessfully set!");
                    break;
                
                case "get":
                    await GetMessage(message.Channel.GuildId, message.Channel, parameter);
                    break;
                
                // Remove the welcome/leave message
                case "clear":
                    bool finishClear = await ClearMessage(message.Channel.GuildId, parameter);
                    if (finishClear)
                        await message.RespondAsync($"`{parameter}` message sucessfully cleared!");
                    else
                        await message.RespondAsync($"`{parameter}` message doesn't exist! Did you not set it?");
                    break;
                
                // Remove the guild from the Database     
                case "clearall":
                case "purge":
                    await ClearMessage(message.Channel.GuildId, "all");
                    await message.RespondAsync($"You have been wiped from the database. \nPlease run the welcome set or leave set command if you want to re-add the messages");
                    break;
            }
        }

        private async Task HandleRoleConfiguration(DiscordMessage message, string parameter)
        {
            Console.WriteLine(_stringRenderer.GetMessageCount(message));

            switch (parameter)
            {
                case "set":
                case "add":
                    // Add any new mentioned roles to give on welcome
                    List<(ulong, bool)> roleIds = await SetRoles(message.Channel.GuildId, message.MentionedRoles);
                    await message.RespondAsync("", false, EmbedStore.NewMemberRolesEmbed(message.Channel.Guild, roleIds, true));
                    break;

                case "remove":
                    // Get a list for removing the roles
                    List<(ulong, bool)> idsToRemove = await PrepForRemoval(message);

                    // Remove the roles from the Database if the admin didn't remove them already
                    await RemoveRoles(message.Channel.GuildId, idsToRemove);
                    await message.RespondAsync("", false, EmbedStore.NewMemberRolesEmbed(message.Channel.Guild, idsToRemove, false));
                    break;
                
                case "get":
                    // Get all welcome roles
                    List<ulong> guildRoleIds = await GetRoles(message.Channel.GuildId);
                    await message.RespondAsync("", false, EmbedStore.NewMemberRolesInfo(message.Channel.Guild, guildRoleIds));
                    break; 

                case "clear":
                    // Purges out all welcome roles from the Database
                    await RemoveAllRoles(message.Channel.GuildId);
                    await message.RespondAsync("Cleared all roles to be assigned on welcome. You will have to re-add them manually.");
                    break;
            }
        }

        private async Task<List<(ulong, bool)>> SetRoles(ulong guildId, IReadOnlyList<DiscordRole> mentionedRoles)
        {
            var result = await _context.WelcomeRoles.Where(q => q.GuildInfoGuildId.Equals(guildId)).ToListAsync();

            List<(ulong Id, bool exists)> roleIds = new List<(ulong Id, bool exists)>();

            // If the role exists in the database, mark it as such
            foreach (var i in mentionedRoles)
            {
                if (await _context.WelcomeRoles.AnyAsync(q => q.RoleId.Equals(i.Id)))
                    roleIds.Add((i.Id, true));
                else
                    roleIds.Add((i.Id, false));
            }

            await AddRoles(guildId, roleIds);

            return roleIds;
        }

        private async Task AddRoles(ulong guildId, List<(ulong Id, bool exists)> roleIds)
        {
            List<ulong> addedRoleIds = new List<ulong>();

            // Add the roles in the addedIds list if the role doesn't already exist in the Database
            foreach (var i in roleIds)
            {
                if (!i.exists)
                    addedRoleIds.Add(i.Id);
            }

            foreach (var j in addedRoleIds)
            {
                WelcomeRole FileToAdd = new WelcomeRole
                {
                    GuildInfoGuildId = guildId,
                    RoleId = j
                };

                await _context.AddAsync(FileToAdd);
            }

            await _context.SaveChangesAsync();
        }

        private async Task<List<(ulong, bool)>> PrepForRemoval(DiscordMessage message)
        {
            List<(ulong Id, bool exists)> idsToRemove = new List<(ulong, bool)>();

            // If the role exists in the Database, mark it as such
            foreach (var i in message.MentionedRoles)
            {
                if (await _context.WelcomeRoles.AnyAsync(q => q.RoleId.Equals(i.Id)))
                    idsToRemove.Add((i.Id, true));
                else
                    idsToRemove.Add((i.Id, false));
            }

            return idsToRemove;
        }

        private async Task RemoveRoles(ulong guildId, List<(ulong Id, bool exists)> idsToRemove)
        {   
            // If the role exists in the database, remove it
            foreach (var i in idsToRemove)
            {
                if (i.exists)
                {
                    var result = await _context.WelcomeRoles
                        .Where(q => q.GuildInfoGuildId.Equals(guildId))
                        .Where(q => q.RoleId.Equals(i.Id))
                        .FirstOrDefaultAsync();
                
                    _context.Remove(result);
                }
            }

            await _context.SaveChangesAsync();
        }

        // Gets rid of all welcome roles
        private async Task RemoveAllRoles(ulong guildId)
        {
            var result = await _context.WelcomeRoles.Where(q => q.GuildInfoGuildId.Equals(guildId)).ToListAsync();

            foreach (var i in result)
            {
                _context.Remove(i);
            }

            await _context.SaveChangesAsync();
        }

        private async Task AssignRoles(DiscordGuild guild, DiscordMember member)
        {
            List<(ulong, bool)> idsToRemove = new List<(ulong, bool)>();
            var roleIds = await _context.WelcomeRoles.Where(q => q.GuildInfoGuildId.Equals(guild.Id)).ToListAsync();

            // If the server has the role, assign it. Otherwise, flag it for removal
            foreach (var i in roleIds)
            {
                if (guild.Roles.ContainsKey(i.RoleId))
                {
                    DiscordRole role = guild.GetRole(i.RoleId);
                    await member.GrantRoleAsync(role);
                }
                else
                    idsToRemove.Add((i.RoleId, true));
            }

            if (idsToRemove.Count > 0)
                await RemoveRoles(guild.Id, idsToRemove);
        }

        // Prints all welcome roles on an embed
        private async Task<List<ulong>> GetRoles(ulong guildId)
        {
            List<ulong> roleIds = new List<ulong>();

            var result = await _context.WelcomeRoles.Where(q => q.GuildInfoGuildId.Equals(guildId)).ToListAsync();

            foreach (var i in result)
            {
                roleIds.Add(i.RoleId);
            }

            return roleIds;
        }

        // Required to register the WelcomeMessage in the database.
        private async Task InitialSetup(ulong guildId, ulong channelId, string welcomeMessage = null, string leaveMessage = null)
        {
            NewMember FileToAdd = new NewMember
            {
                GuildInfoGuildId = guildId,
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
        private async Task<bool> SetChannel(ulong guildId, ulong channelId)
        {
            var result = await _context.NewMembers
                .FirstOrDefaultAsync(q => q.GuildInfoGuildId.Equals(guildId));
            
            if (result.ChannelId == 0)
                return false;

            result.ChannelId = channelId;
            await _context.SaveChangesAsync();

            return true;
        }

        /*
         * Set the welcome/leave message
         * Welcome/leave toggle is decided by the parameter argument
         */
        private async Task SetMessage(ulong guildId, ulong channelId, string newMessage, string parameter)
        {
            if (!await _guildInfoHelper.EnsureGuild(guildId))
                _guildInfoHelper.AddNewEntry(guildId);

            var result = await _context.NewMembers
                .FirstOrDefaultAsync(q => q.GuildInfoGuildId.Equals(guildId));

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
        private async Task<bool> ClearMessage(ulong guildId, string parameter)
        {
            var result = await _context.NewMembers
                .FirstOrDefaultAsync(q => q.GuildInfoGuildId.Equals(guildId));
            
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

         //Get the Welcome/Leave message
        private async Task GetMessage(ulong guildId, DiscordChannel channel, string parameter)
        {
            var result = await _context.NewMembers
                .FirstOrDefaultAsync(q => q.GuildInfoGuildId.Equals(guildId));

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

        public static async Task NewMemberRolesHelp(DiscordMessage message)
        {
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();

            eb.WithTitle("Welcome roles subcategory");
            eb.WithDescription("Gives roles when a user joins the server (subcommand of welcome)");
            eb.AddField("SubCommands", "set <role mention>: Sets the roles to give the user on a welcome event. Make sure they're mentionable! Can add more than one mention. \n\n" +
                                        "remove <role mention>: Removes a role given on welcome. \n\n" +
                                        "clear: Removes all roles given on welcome. \n\n" +
                                        "get: Prints out all roles given on welcome.");

            await message.RespondAsync("", false, eb.Build());
        }
    }
}
