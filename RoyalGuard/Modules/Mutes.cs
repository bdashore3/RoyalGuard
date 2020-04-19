using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using System.Linq;
using DSharpPlus;
using DSharpPlus.Entities;
using Microsoft.EntityFrameworkCore;
using RoyalGuard.Helpers;
using RoyalGuard.Helpers.Data;

namespace RoyalGuard.Modules
{
    public class Mutes
    {
        private readonly RoyalGuardContext _context;
        public Mutes(RoyalGuardContext context)
        {
            _context = context;
        }

        public async Task MuteUser(DiscordMessage message)
        {
            DiscordRole muteRole = await HandleMuteRole(message);
            DiscordMember member = message.MentionedUsers[0] as DiscordMember;

            if (member.Roles.Contains(muteRole))
            {
                await message.RespondAsync("This user is already muted!");
                return;
            }

            await member.GrantRoleAsync(muteRole);

            await message.RespondAsync("", false, EmbedStore.GetMuteEmbed(message.MentionedUsers[0].AvatarUrl, message.MentionedUsers[0].Username, true));
        }

        public async Task UnmuteUser(DiscordMessage message)
        {
            DiscordRole muteRole = await HandleMuteRole(message);
            DiscordMember member = message.MentionedUsers[0] as DiscordMember;

            if (!member.Roles.Contains(muteRole))
            {
                await message.RespondAsync("This user is not muted!");
                return;
            }
            
            await member.RevokeRoleAsync(muteRole);

            await message.RespondAsync("", false, EmbedStore.GetMuteEmbed(message.MentionedUsers[0].AvatarUrl, message.MentionedUsers[0].Username, false));
        }

        public async Task<DiscordRole> HandleMuteRole(DiscordMessage message)
        {
            var result = await _context.GuildRoleStore
                .FirstOrDefaultAsync(q => q.GuildId.Equals(message.Channel.GuildId));
            
            if (result == null)
            {
                await message.RespondAsync("Created a new role called `Muted`. \nIf you accidentally delete this role, a new one will be created \nAll channels have been updated with the mute role");
                return await NewMuteRole(message.Channel.Guild);
            }

            else if (!message.Channel.Guild.Roles.ContainsKey(result.MutedRoleId))
            {
                await message.RespondAsync("You deleted the mute role from your server, but the database wasn't updated! Recreating role");
                _context.Remove(result);
                return await NewMuteRole(message.Channel.Guild);
            }

            else
                return message.Channel.Guild.GetRole(result.MutedRoleId);
        }

        public async Task<DiscordRole> NewMuteRole(DiscordGuild guild)
        {
            DiscordRole muteRole = await guild.CreateRoleAsync("muted", Permissions.AccessChannels | Permissions.ReadMessageHistory);
            foreach (var entry in guild.Channels)
                await entry.Value.AddOverwriteAsync(muteRole, Permissions.None, Permissions.SendMessages | Permissions.SendTtsMessages);

            GuildRole FileToAdd = new GuildRole
            {
                GuildId = guild.Id,
                MutedRoleId = muteRole.Id
            };

            await _context.AddAsync(FileToAdd);
            await _context.SaveChangesAsync();

            return muteRole;
        }

        public static async Task MuteHelp(DiscordMessage message)
        {
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();
            eb.WithTitle("Mute Help");
            eb.WithDescription("Description: Commands for muting users in a server");
            eb.AddField("Commands", "mute <mention>: Mutes the mentioned user. Creates a role if it doesn't exist \n\n" +
                                    "unmute <mention>: Unmutes the mentioned user \n\n");

            await message.RespondAsync("", false, eb.Build());
        }
    }
}
