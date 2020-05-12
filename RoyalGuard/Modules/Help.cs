using System;
using System.Threading.Tasks;
using DSharpPlus.Entities;
using RoyalGuard.Handlers;
using RoyalGuard.Helpers.Commands;
using RoyalGuard.Helpers.Security;

namespace RoyalGuard.Modules
{
    public class Help
    {
        // Variables and constructor for DI
        private readonly StringRenderer _stringRenderer;
        public Help(StringRenderer stringRenderer)
        {
            _stringRenderer = stringRenderer;
        }

        // Figure out what aspect of help the user needs
        public async Task DirectHelp(DiscordMessage message)
        {
            if (_stringRenderer.GetMessageCount(message) < 2)
            {
                await SendGenericHelp(message);
                return;
            }

            string instruction = _stringRenderer.GetWordFromIndex(message, 1);

            switch(instruction)
            {
                case "warn":
                    await Warns.WarnHelp(message);
                    break;

                case "ban":
                    await Bans.BanHelp(message);
                    break;

                case "mute":
                    await Mutes.MuteHelp(message);
                    break;

                case "welcome":
                case "leave":
                    await NewMemberHandler.NewMemberHelp(message);
                    break;
                
                case "welcomeroles":
                    await NewMemberHandler.NewMemberRolesHelp(message);
                    break;

                case "prefix":
                    await PrefixHelper.PrefixHelp(message);
                    break;
            }
        }

        // Generic help if there's nothing after the help command
        private async Task SendGenericHelp(DiscordMessage message)
        {
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();

            eb.WithTitle("RoyalGuard Help");
            eb.WithDescription("Help for the RoyalGuard Discord bot");
            eb.AddField("Subcategories", "```\n" +
                                        "ban \n" +
                                        "warn \n" +
                                        "mute \n" +
                                        "welcome \n" +
                                        "leave \n" +
                                        "welcomeroles \n" +
                                        "prefix \n" +
                                        "purge" +
                                        "```");

            await message.RespondAsync("", false, eb.Build());
        }

        public async Task SendEmergencyHelp(DiscordMessage message)
        {
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();

            eb.WithTitle("RoyalGuard Emergency Help");
            eb.WithDescription("You should only use this if you mess up your prefix!");
            eb.AddField("Commands", "prefix <new prefix>: Changes the prefix for the server \n\n" +
                                    $"resetprefix: Resets the prefix to {CredentialsHelper.DefaultPrefix} \n\n");
            
            await message.RespondAsync("", false, eb.Build());
        }

        public async Task SendSupportMessage(DiscordMessage message)
        {
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();

            eb.WithTitle("RoyalGuard Support");
            eb.WithDescription("Need more help?");
            eb.AddField("Support server", "https://discord.gg/pswt7by");
            eb.AddField("kingbri's twitter", "https://twitter.com/kingbri_aahs");
            eb.WithFooter("Created with ❤️ by kingbri#6666");

            await message.RespondAsync("", false, eb.Build());
        }
    }
}
