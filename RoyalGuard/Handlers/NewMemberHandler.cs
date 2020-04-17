using System.Threading.Tasks;
using DSharpPlus.Entities;
using Microsoft.EntityFrameworkCore;
using RoyalGuard.Helpers.Commands;
using RoyalGuard.Helpers.Data;
using RoyalGuard.Helpers.Security;

namespace RoyalGuard.Handlers
{
    public class NewMemberHandler
    {
        private readonly RoyalGuardContext _context;
        private readonly StringRenderer _stringRenderer;
        public NewMemberHandler(RoyalGuardContext context, StringRenderer stringRenderer)
        {
            _context = context;
            _stringRenderer = stringRenderer;
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
                    if (result.LeaveMessage == null)
                        return; 

                    message = result.LeaveMessage
                        .Replace("{member}", member)
                        .Replace("{user}", member)
                        .Replace("{server}", guild.Name);

                    break;
                
                case "welcome":
                    if (result.WelcomeMessage == null)
                        return;

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

            switch(instruction)
            {
                case "init":
                case "initialsetup":
                    await InitialSetup(message.Channel.GuildId, message.ChannelId);
                    await message.RespondAsync(
                        $"Sucessfully initalized your messages! \nConfigure them using `{CredentialsHelper.Prefix}welcome set` or `{CredentialsHelper.Prefix}leave set`!");
                    break;
                
                case "channel":
                    await SetChannel(message.Channel.GuildId, message.ChannelId);
                    await message.RespondAsync($"`{parameter}` channel sucessfully set!");
                    break;
                
                case "set":
                    string newMessage = _stringRenderer.RemoveExtras(message, 2);
                    await SetMessage(message.Channel.GuildId, newMessage, parameter);
                    await message.RespondAsync($"`{parameter}` message sucessfully set!");
                    break;
                
                case "clear":
                    await ClearMessage(message.Channel.GuildId, parameter);
                    await message.RespondAsync($"`{parameter}` message sucessfully cleared!");
                    break;
                
                case "clearall":
                case "purge":
                    await ClearMessage(message.Channel.GuildId, "all");
                    await message.RespondAsync($"You have been wiped from the database. Please run `{CredentialsHelper.Prefix}init` if you want to re-add the messages");
                    break;
            }
        }

        public async Task InitialSetup(ulong guildId, ulong channelId)
        {
            NewMember FileToAdd = new NewMember
            {
                GuildId = guildId,
                ChannelId = channelId,
                WelcomeMessage = null,
                LeaveMessage = null
            };

            await _context.AddAsync(FileToAdd);
            await _context.SaveChangesAsync();
        }

        public async Task SetChannel(ulong guildId, ulong channelId)
        {
            var result = await _context.NewMembers
                .FirstOrDefaultAsync(q => q.GuildId.Equals(guildId));
            
            result.ChannelId = channelId;
            await _context.SaveChangesAsync();
        }

        public async Task SetMessage(ulong guildId, string newMessage, string parameter)
        {
            var result = await _context.NewMembers
                .FirstOrDefaultAsync(q => q.GuildId.Equals(guildId));

            switch (parameter)
            {
                case "leave":
                    result.LeaveMessage = newMessage;
                    break;

                case "welcome":
                    result.WelcomeMessage = newMessage;
                    break;
            }

            await _context.SaveChangesAsync();
        }

        public async Task ClearMessage(ulong guildId, string parameter)
        {
            var result = await _context.NewMembers
                .FirstOrDefaultAsync(q => q.GuildId.Equals(guildId));
            
            switch (parameter)
            {
                case "leave":
                    result.LeaveMessage = null;
                    break;
                
                case "welcome":
                    result.WelcomeMessage = null;
                    break;
                
                case "all":
                    _context.Remove(result);
                    break;
            }

            await _context.SaveChangesAsync();
        }

        public async Task GetMessage(ulong guildId, DiscordChannel channel, string parameter)
        {
            var result = await _context.NewMembers
                .FirstOrDefaultAsync(q => q.GuildId.Equals(guildId));

            switch (parameter)
            {
                case "leave":
                    await channel.SendMessageAsync(result.LeaveMessage);
                    break;
                
                case "welcome":
                    await channel.SendMessageAsync(result.WelcomeMessage);
                    break;
            }
        }
    }
}
