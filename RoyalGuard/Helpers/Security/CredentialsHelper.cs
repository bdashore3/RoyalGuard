using System;
using System.IO;
using System.Text.Json;

namespace RoyalGuard.Helpers.Security
{
    public class CredentialsHelper
    {
        public static string DefaultPrefix;
        public static ulong BotId;

        // Read credentials from the JSON
        public BotInformation ReadCreds(string path)
        {
            string infoString = File.ReadAllText(path);
            return JsonSerializer.Deserialize<BotInformation>(infoString);
        }

        // Set all static variables used in the bot
        public void SetStatics(string path)
        {
            var info = ReadCreds(path);
            DefaultPrefix = info.DefaultPrefix;
            BotId = UInt64.Parse(info.BotIdString);
        }

        // Get the db connection string from the provided path
        public static string GetConnectionString(string path)
        {
            CredentialsHelper helper = new CredentialsHelper();
            var info = helper.ReadCreds(path);
            return info.DBConnection;
        }

        // Get the discord bot token
        public string GetBotToken(string path)
        {
            var info = ReadCreds(path);
            return info.BotToken;
        }
    }

    public class BotInformation
    {
        public string BotIdString { get; set; }
        public string BotToken { get; set; }
        public string DefaultPrefix { get; set; }
        public string DBConnection { get; set; }
    }
}
