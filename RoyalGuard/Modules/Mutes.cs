using System;
using System.Threading.Tasks;
using System.Linq;
using DSharpPlus;
using DSharpPlus.Entities;
using Microsoft.EntityFrameworkCore;
using RoyalGuard.Helpers;
using RoyalGuard.Helpers.Data;
using RoyalGuard.Handlers;
using RoyalGuard.Helpers.Commands;
using System.Timers;

namespace RoyalGuard.Modules
{
    public class Mutes
    {
        private readonly RoyalGuardContext _context;
        private readonly StringRenderer _stringRenderer;
        private readonly TimeConversion _timeConversion;
        private readonly TrieHandler _trieHandler;
        private readonly PermissionsHandler _permissionsHandler;
        private readonly GuildInfoHelper _guildInfoHelper;
        public Mutes(
            RoyalGuardContext context, 
            StringRenderer stringRenderer, 
            TimeConversion timeConversion, 
            TrieHandler trieHandler,
            PermissionsHandler permissionsHandler,
            GuildInfoHelper guildInfoHelper)
        {
            _context = context;
            _stringRenderer = stringRenderer;
            _timeConversion = timeConversion;
            _trieHandler = trieHandler;
            _permissionsHandler = permissionsHandler;
            _guildInfoHelper = guildInfoHelper;
        }

        // Primary function to mute a user
        public async Task MuteUser(DiscordMessage message)
        {
            string stringMuteTimeDiff = null;
            bool usingTime = false;

            if (_stringRenderer.GetMessageCount(message) < 2)
            {
                await MuteHelp(message);
                return;
            }

            if (message.MentionedUsers.Count == 0)
            {
                await message.RespondAsync("Please mention the user you want to mute!");
                return;
            }

            // Get the role for muting a user and convert the mentioned user to a DiscordMember
            DiscordRole muteRole = await HandleMuteRole(message.Channel.Guild, message.Channel);
            DiscordMember member = message.MentionedUsers[0] as DiscordMember;

            Console.WriteLine(_stringRenderer.GetMessageCount(message));

            // Don't mute administrators
            if (_permissionsHandler.CheckAdminFromMention(message.MentionedUsers[0], message.Channel))
            {
                await message.RespondAsync("I cannot mute an administrator! Please demote the user and try again.");
                return;
            }

            // Check if the user is already muted
            if (member.Roles.Contains(muteRole))
            {
                await message.RespondAsync("This user is already muted!");
                return;
            }

            // Give the user the mute role
            await member.GrantRoleAsync(muteRole);

            /*
             * Get the time type (hours, minutes, seconds, days, weeks)
             * Convert the second word into ms for the timer
             */

            // TODO: Make this check see if the character is in a list
            if (_stringRenderer.GetMessageCount(message) >= 3)
            {
                stringMuteTimeDiff = _stringRenderer.GetWordFromIndex(message, 2);
                string timeTypeString = stringMuteTimeDiff.Substring(stringMuteTimeDiff.Length - 1);

                if (timeTypeString.Equals("w") || timeTypeString.Equals("d") || timeTypeString.Equals("h") || timeTypeString.Equals("m") || timeTypeString.Equals("s"))
                {
                    usingTime = true;
                    string timeType = stringMuteTimeDiff.Substring(stringMuteTimeDiff.Length - 1);
                    long muteTimeNum = Int64.Parse(stringMuteTimeDiff.Remove(stringMuteTimeDiff.Length - 1));

                    long muteTimeDiff = _timeConversion.ConvertTime(muteTimeNum, timeType);

                    await AddMuteTime(message.Channel.GuildId, message.MentionedUsers[0].Id, muteTimeDiff);
                }
                else
                    await message.RespondAsync("Please enter the correct syntax for a timed mute! Check the help for more information");
            }

            await message.RespondAsync
                ("", false, EmbedStore.GetMuteEmbed(message.MentionedUsers[0].AvatarUrl, message.MentionedUsers[0].Username, true, usingTime, stringMuteTimeDiff));
        }

        public async Task UnmuteUser(DiscordMessage message)
        {
            if (_stringRenderer.GetMessageCount(message) < 2)
            {
                await MuteHelp(message);
                return;
            }

            if (message.MentionedUsers.Count == 0)
            {
                await message.RespondAsync("Please mention the user you want to unmute!");
                return;
            }

            DiscordRole muteRole = await HandleMuteRole(message.Channel.Guild, message.Channel);
            DiscordMember member = message.MentionedUsers[0] as DiscordMember;

            // Check if the user doesn't have the mute role
            if (!member.Roles.Contains(muteRole))
            {
                await message.RespondAsync("This user is not muted!");
                return;
            }
            
            // Remove the mute role first
            await member.RevokeRoleAsync(muteRole);

            // If a timer exists for a user, stop the timer, remove the mute, and remove mute database entries
            if (_trieHandler.RetrieveMute(message.Channel.GuildId, message.MentionedUsers[0].Id))
            {
                await message.RespondAsync($"Stopping mute timer for {member.Username}");
                _trieHandler.StopMuteTimer(message.Channel.GuildId, member.Id);
                _trieHandler.RemoveExistingMute(message.Channel.GuildId, member.Id);
                await UnmuteUserByTime(message.Channel.GuildId, member.Id, false);
            }

            await message.RespondAsync("", false, EmbedStore.GetMuteEmbed(member.AvatarUrl, member.Username, false, false));
        }
        public async Task AddMuteTime(ulong guildId, ulong userId, long muteTimeDiff)
        {
            // Take the current time and add it to the provided difference
            long curTime = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();
            long muteTime = curTime + muteTimeDiff;

            // Add the total muteTime in the database
            Mute FileToAdd = new Mute
            {
                GuildId = guildId,
                UserId = userId,
                MuteTime = muteTime
            };

            await _context.AddAsync(FileToAdd);
            await _context.SaveChangesAsync();

            // If a mute timer doesn't exist already, create and start a new timer on a new thread
            if (!_trieHandler.RetrieveMute(guildId, userId))
            {
                _ = Task.Run(() => CreateMuteTimer(guildId, userId, muteTimeDiff));
            }
        }

        /*
         * Creates a new mute timer, and starts it.
         * The timer is then added to the mute trie for caching and later reference
         *
         * If there is no guild entry, make a new one in the triehandler and re-run the function
         */
        public void CreateMuteTimer(ulong guildId, ulong userId, long muteTimeDiff)
        {
            Timer muteTimer = new Timer(muteTimeDiff);
            muteTimer.Elapsed += (sender, e) => OnTimedEvent(sender, guildId, userId);
            muteTimer.AutoReset = true;
            muteTimer.Start();

            if (!_trieHandler.AddNewMute(guildId, userId, muteTimer))
                _trieHandler.AddNewMute(guildId, userId, muteTimer);
        }

        // Stop the new timer on a different CPU thread and remove mute entries
        public async void OnTimedEvent(Object source, ulong guildId, ulong userId)
        {
            _trieHandler.StopMuteTimer(guildId, userId);
            _trieHandler.RemoveExistingMute(guildId, userId);
            await UnmuteUserByTime(guildId, userId, true);
        }

        /*
         * Construct a guild, member, and channel object
         * MuteChannel is taken from the Database because the bot can't send a message otherwise
         * The MuteChannel is set when the admin creates the mute role for the first time
         *
         * Once we have those parameters, we can unmute the user and send a message
         */
        public async Task UnmuteUserByTime(ulong guildId, ulong userId, bool sendMessage)
        {
            DiscordGuild guild;

            var result = await _context.Mutes
                .Where(q => q.GuildId.Equals(guildId))
                .Where(q => q.UserId.Equals(userId))
                .FirstOrDefaultAsync();

            guild = await DiscordBot.discord.GetGuildAsync(guildId);
            DiscordMember member = await guild.GetMemberAsync(userId);
            DiscordRole muteRole = await HandleMuteRole(guild);
            DiscordChannel muteChannel = guild.GetChannel(await GetMuteChannel(guildId));

            await member.RevokeRoleAsync(await HandleMuteRole(guild));
            _context.Remove(result);
            await _context.SaveChangesAsync();

            if (sendMessage)
                await muteChannel.SendMessageAsync("", false, EmbedStore.GetMuteEmbed(member.AvatarUrl, member.Username, false, false));
        }

        /*
         * If the mute role doesn't exist in the database and the server, create a new one
         * If the mute role is deleted from the server, update the database
         * Otherwise, return the existing mute role
         */
        public async Task<DiscordRole> HandleMuteRole(DiscordGuild guild, DiscordChannel channel = null)
        {
            if (!await _guildInfoHelper.EnsureGuild(guild.Id))
                _guildInfoHelper.AddNewEntry(guild.Id);

            var result = await _context.GuildInfoStore
                .FirstOrDefaultAsync(q => q.GuildId.Equals(guild.Id));

            if (channel == null)
            {
                channel = guild.GetChannel(result.MuteChannelId);
            }

            if (result == null || result.MutedRoleId == 0)
            {
                await channel.SendMessageAsync("Created a new role called `Muted`. \n" +
                                                "If you accidentally delete this role, a new one will be created \n" +
                                                "All channels have been updated with the mute role \n" +
                                                "Use `mutechannel` to change where timed unmutes are sent");
                return await NewMuteRole(guild, channel.Id);
            }

            else if (!guild.Roles.ContainsKey(result.MutedRoleId))
            {
                await channel.SendMessageAsync("You deleted the mute role from your server, but the database wasn't updated! Recreating role");
                return await NewMuteRole(guild, channel.Id);
            }

            else
                return guild.GetRole(result.MutedRoleId);
        }

        // Create a new mute role and save the ID in the database
        public async Task<DiscordRole> NewMuteRole(DiscordGuild guild, ulong muteChannelId)
        {
            DiscordRole muteRole = await guild.CreateRoleAsync("muted", Permissions.AccessChannels | Permissions.ReadMessageHistory);
            foreach (var entry in guild.Channels)
                await entry.Value.AddOverwriteAsync(muteRole, Permissions.None, Permissions.SendMessages | Permissions.SendTtsMessages);
            
            var result = await _context.GuildInfoStore
                .FirstOrDefaultAsync(q => q.GuildId.Equals(guild.Id));

            result.MutedRoleId = muteRole.Id;
            result.MuteChannelId = muteChannelId;

            await _context.SaveChangesAsync();

            return muteRole;
        }

        // Update the mute channels and roles
        public async Task UpdateRoles(ulong guildId, ulong mutedRoleId, ulong muteChannelId)
        {
            var result = await _context.GuildInfoStore
                .FirstOrDefaultAsync(q => q.GuildId.Equals(guildId));
            
            result.MutedRoleId = mutedRoleId;
            result.MuteChannelId = muteChannelId;

            await _context.SaveChangesAsync();
        }

        // Gets the channel to send mute messages if not provided
        public async Task<ulong> GetMuteChannel(ulong guildId)
        {
            var result = await _context.GuildInfoStore
                .FirstOrDefaultAsync(q => q.GuildId.Equals(guildId));
            
            return result.MuteChannelId;
        }

        // Changes where mute messages are sent
        public async Task ChangeMuteChannel(DiscordMessage message)
        {
            ulong muteChannelId = message.MentionedChannels[0].Id;
            var result = await _context.GuildInfoStore
                .FirstOrDefaultAsync(q => q.GuildId.Equals(message.Channel.GuildId));

            result.MuteChannelId = muteChannelId;
            await message.RespondAsync("" , false, EmbedStore.ChannelEmbed("Mute", muteChannelId));
            await _context.SaveChangesAsync();
        }

        /* 
         * Load all of the saved mute timers at startup
         * If the difference between the stored time and the current time is negative,
         * remove the database entry
         */
        public async Task LoadMuteTimers()
        {
            var result = await _context.Mutes.ToListAsync();

            foreach (var i in result)
            {
                long muteTimeDiff = i.MuteTime - DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();

                if (muteTimeDiff <= 0)
                {
                    _context.Remove(i);
                    await _context.SaveChangesAsync();
                    break;                    
                }

                _ = Task.Run(() => CreateMuteTimer(i.GuildId, i.UserId, muteTimeDiff));
            }
        }

        // Modular help for mutes
        public static async Task MuteHelp(DiscordMessage message)
        {
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();
            eb.WithTitle("Mute Help");
            eb.WithDescription("Description: Commands for muting users in a server");
            eb.AddField("Commands", "mute <mention> <time(w, d, h, m, s)>: Mutes the mentioned user. Creates a role if it doesn't exist. If the time is provided, the user will be muted for a period of time \n\n" +
                                    "unmute <mention>: Unmutes the mentioned user. Overrides all time based mutes \n\n" +
                                    "mutechannel <channel Id>: Sets the channel where timed unmutes are sent. This is where the mute role is created by default");

            await message.RespondAsync("", false, eb.Build());
        }
    }
}
