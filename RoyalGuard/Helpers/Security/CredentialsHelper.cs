using System;
using System.IO;
using Newtonsoft.Json;

namespace RoyalGuard.Helpers.Security
{
    public class CredentialsHelper
    {
        // All variables are initialized here
        public static string BotToken { get; private set; }
        public static string DefaultPrefix { get; private set; }
        public static string DBConnection { get; private set; }

        // This struct might show warnings about no initialized value
        // It is assigned by the JSON read operation in ReadCreds()
#pragma warning disable 0649
        private struct CredsJson
        { 
            [JsonProperty("BotToken")]
            public string BotToken;

            [JsonProperty("DefaultPrefix")]
            public string DefaultPrefix;

            [JsonProperty("DBConnection")]
            public string DBConnection;
        }
#pragma warning restore 0649
        public static bool ReadCreds(string path)
        {
            // Read credentials as Token and DevID into a struct object from creds.json
            string info = "";
            using (FileStream fs = File.OpenRead(path))
            using (StreamReader sr = new StreamReader(fs))
                info = sr.ReadToEnd();

            CredsJson creds = JsonConvert.DeserializeObject<CredsJson>(info);
            BotToken = creds.BotToken;
            DefaultPrefix = creds.DefaultPrefix;
            DBConnection = creds.DBConnection;
            return true;
        }

        // Empty the tokens from RAM once we've authenticated
        public static void WipeToken()
        {
            BotToken = "";
        }
    }
}
