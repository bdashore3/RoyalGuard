using System;
using Microsoft.EntityFrameworkCore;

namespace RoyalGuard.Helpers.Data
{
    public class RoyalGuardContext : DbContext
    {
        public RoyalGuardContext(DbContextOptions<RoyalGuardContext> options) : base(options)
        {
        }
        public DbSet<Mute> Mutes { get; set; }
        public DbSet<Warn> Warns { get; set; }
        public DbSet<NewMember> NewMembers { get; set; }
        public DbSet<CustomPrefix> CustomPrefixes { get; set; }
    }
    public class Mute
    {
        public Guid Id { get; set; }
        public ulong GuildId { get; set; }
        public ulong UserId { get; set; }
        public long MuteTime { get; set; }
    }
    public class Warn
    {
        public Guid Id { get; set; }
        public ulong GuildId { get; set; }
        public ulong UserId { get; set; }
        public int WarnNumber { get; set; }
    }

    public class NewMember
    {
        public Guid Id { get; set; }
        public ulong GuildId { get; set; }
        public ulong ChannelId { get; set; }
        public string WelcomeMessage { get; set; }
        public string LeaveMessage { get; set; }
    }

    public class CustomPrefix
    {
        public Guid Id { get; set; }
        public ulong GuildId { get; set; }
        public string Prefix { get; set; }
    }
}
