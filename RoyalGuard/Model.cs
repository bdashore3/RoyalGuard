using System;
using System.Collections.Generic;
using Microsoft.EntityFrameworkCore;

namespace RoyalGuard.Helpers.Data
{
    public class RoyalGuardContext : DbContext
    {
        public RoyalGuardContext(DbContextOptions<RoyalGuardContext> options) : base(options)
        {
        }

        protected override void OnModelCreating(ModelBuilder modelBuilder)
        {
            modelBuilder.Entity<GuildInfo>()
                .HasMany(f => f.WarnCollection)
                .WithOne(f => f.GuildInfo)
                .OnDelete(DeleteBehavior.Cascade);
            
            modelBuilder.Entity<GuildInfo>()
                .HasMany(f => f.NewMemberCollection)
                .WithOne(f => f.GuildInfo)
                .OnDelete(DeleteBehavior.Cascade);
            
            modelBuilder.Entity<GuildInfo>()
                .HasMany(f => f.DeleteTimeCollection)
                .WithOne(f => f.GuildInfo)
                .OnDelete(DeleteBehavior.Cascade);
            
            modelBuilder.Entity<GuildInfo>()
                .HasMany(f => f.WelcomeRoleCollection)
                .WithOne(f => f.GuildInfo)
                .OnDelete(DeleteBehavior.Cascade);
            
            modelBuilder.Entity<GuildInfo>()
                .HasKey(f => f.GuildId);
        }

        public DbSet<Mute> Mutes { get; set; }
        public DbSet<GuildInfo> GuildInfoStore { get; set; }
        public DbSet<Warn> Warns { get; set; }
        public DbSet<NewMember> NewMembers { get; set; }
        public DbSet<DeleteTimeInfo> DeleteTimeStore { get; set; }
        public DbSet<WelcomeRole> WelcomeRoles { get; set; }
    }
    public class Mute
    {
        public Guid Id { get; set; }
        public ulong GuildId { get; set; }
        public ulong UserId { get; set; }
        public long MuteTime { get; set; }
    }
    public class GuildInfo
    {
        public ulong GuildId { get; set; }
        public string Prefix { get; set; }
        public ulong MutedRoleId { get; set; }
        public ulong MuteChannelId { get; set; }
        public ICollection<Warn> WarnCollection { get; set; }
        public ICollection<NewMember> NewMemberCollection { get; set; }
        public ICollection<DeleteTimeInfo> DeleteTimeCollection { get; set; }
        public ICollection<WelcomeRole> WelcomeRoleCollection { get; set; }
    }
    public class DeleteTimeInfo
    {
        public Guid Id { get; set; }
        public ulong GuildInfoGuildId { get; set; }
        public long DeleteTime { get; set; }
        public GuildInfo GuildInfo { get; set; }
    }
    public class Warn
    {
        public Guid Id { get; set; }
        public ulong GuildInfoGuildId { get; set; }
        public ulong UserId { get; set; }
        public int WarnNumber { get; set; }
        public GuildInfo GuildInfo { get; set; }
    }

    public class NewMember
    {
        public Guid Id { get; set; }
        public ulong GuildInfoGuildId { get; set; }
        public ulong ChannelId { get; set; }
        public string WelcomeMessage { get; set; }
        public string LeaveMessage { get; set; }
        public List<ulong> RolesList { get; set; }
        public GuildInfo GuildInfo { get; set; }
    }

    public class WelcomeRole
    {
        public Guid Id { get; set; }
        public ulong GuildInfoGuildId { get; set; }
        public ulong RoleId { get; set; }
        public GuildInfo GuildInfo { get; set; }
    }
}
