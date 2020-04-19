using System;
using System.Threading.Tasks;
using DSharpPlus.Entities;
using RoyalGuard.Handlers;
using RoyalGuard.Helpers.Commands;

namespace RoyalGuard.Modules
{
    public class Help
    {
        private readonly StringRenderer _stringRenderer;
        public Help(StringRenderer stringRenderer)
        {
            _stringRenderer = stringRenderer;
        }
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

                case "prefix":
                    await PrefixHelper.PrefixHelp(message);
                    break;
            }
        }

        public async Task SendGenericHelp(DiscordMessage message)
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
                                        "prefix \n" +
                                        "```");

            await message.RespondAsync("", false, eb.Build());
        }
    }
}
