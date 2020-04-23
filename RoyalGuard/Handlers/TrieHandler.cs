using System;
using System.Threading.Tasks;
using System.Timers;
using KTrie;
using RoyalGuard.Helpers.Security;
using RoyalGuard.Modules;

namespace RoyalGuard.Handlers 
{
    /* 
     * Class used inside the globalTrie
     * Contains internal functions for tries inside the globalTrie
     */ 
    public class CachedNode 
    {
        public readonly StringTrie<Timer> mutedUsers;
        public string prefix;

        public CachedNode() 
        {
            mutedUsers = new StringTrie<Timer>();
        }

        public void AddMute(ulong userId, Timer muteTimer) 
        {
            mutedUsers.Add(userId.ToString(), muteTimer);
        }

        public void RemoveMute(ulong userId)
        {
            mutedUsers.Remove(userId.ToString());
        }

        public bool EnsureMute(ulong userId)
        {
            Timer muteTimer;

            if (mutedUsers.TryGetValue(userId.ToString(), out muteTimer))
                return true;
            
            return false;
        }

        public void StopTimer(ulong userId)
        {
            Timer muteTimer;

            if (mutedUsers.TryGetValue(userId.ToString(), out muteTimer))
                muteTimer.Stop();
        }
    }

    /*
     * Various functions that require the use of globalTrie
     *
     * The gatekeeper for CachedNode
     *
     * You should always use this class rather than directly
     * referencing CachedNode.
     */
    public class TrieHandler 
    {
        public StringTrie<CachedNode> globalTrie;
        public TrieHandler()
        {
            globalTrie = new StringTrie<CachedNode>();
        }

        public void AddToTrie(ulong guildId, string prefix)
        {
            CachedNode node;

            if (globalTrie.TryGetValue(guildId.ToString(), out node))
            {
                node.prefix = prefix;
                return;
            }
            
            node = new CachedNode();
            node.prefix = prefix;
            globalTrie.Add(guildId.ToString(), node);
        }

        public string GetPrefix(ulong guildId)
        {
            CachedNode node;

            if (globalTrie.TryGetValue(guildId.ToString(), out node))
                return node.prefix;

            return CredentialsHelper.DefaultPrefix;
        }

        public void AddNewMute(ulong guildId, ulong userId, Timer muteTimer)
        {
            CachedNode node;

            if (globalTrie.TryGetValue(guildId.ToString(), out node))
                node.AddMute(userId, muteTimer);
        }

        public bool RetrieveMute(ulong guildId, ulong userId)
        {
            CachedNode node;

            if (globalTrie.TryGetValue(guildId.ToString(), out node))
            {
                if (node.EnsureMute(userId))
                    return true;
                
                return false;
            }

            return false;
        }

        public void RemoveExistingMute(ulong guildId, ulong userId)
        {
            CachedNode node;

            if (globalTrie.TryGetValue(guildId.ToString(), out node))
                node.RemoveMute(userId);
        }

        public void StopMuteTimer(ulong guildId, ulong userId)
        {
            CachedNode node;

            if (globalTrie.TryGetValue(guildId.ToString(), out node))
                node.StopTimer(userId);
        }
    }
}