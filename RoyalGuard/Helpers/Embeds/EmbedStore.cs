using System;
using DSharpPlus.Entities;

namespace RoyalGuard.Helpers
{
    public class EmbedStore
    {
        public static DiscordEmbed GetBanEmbed(string avatarUrl, string username, string reason)
        {
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();

            eb.WithColor(DiscordColor.Red);
            eb.WithTitle("New Ban");
            eb.WithThumbnailUrl(avatarUrl);
            eb.AddField("Username ", username);
            eb.AddField("Reason", reason);

            return eb.Build();
        }

        public static DiscordEmbed GetUnbanEmbed(string username, bool useId)
        {
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();

            eb.WithColor(DiscordColor.Green);

            if (useId)
                eb.WithTitle("New Unban by ID");
            else
                eb.WithTitle("New Unban");

            eb.WithDescription($"Username: {username}");

            return eb.Build();
        }

        public static DiscordEmbed GetWarnEmbed(string avatarUrl, string username, string warnNumberSend, bool newWarn)
        {
            DiscordEmbedBuilder eb = new DiscordEmbedBuilder();

            if (newWarn)
            {
                eb.WithColor(DiscordColor.IndianRed);
                eb.WithTitle("New Warn");
            }
            else
            {
                eb.WithColor(DiscordColor.Green);
                eb.WithTitle("Removed Warn");
            }
            eb.WithThumbnailUrl(avatarUrl);
            eb.AddField("Username ", username);
            eb.AddField("Warn Amount ",  warnNumberSend);

            return eb.Build();
        }
    }
}
