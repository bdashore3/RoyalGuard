using System;
using Microsoft.EntityFrameworkCore.Migrations;

namespace RoyalGuard.Migrations
{
    public partial class RenameGuildStore : Migration
    {
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "GuildRoleStore");

            migrationBuilder.CreateTable(
                name: "GuildInfoStore",
                columns: table => new
                {
                    Id = table.Column<Guid>(nullable: false),
                    GuildId = table.Column<decimal>(nullable: false),
                    MutedRoleId = table.Column<decimal>(nullable: false),
                    MuteChannelId = table.Column<decimal>(nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_GuildInfoStore", x => x.Id);
                });
        }

        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "GuildInfoStore");

            migrationBuilder.CreateTable(
                name: "GuildRoleStore",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    GuildId = table.Column<decimal>(type: "numeric(20,0)", nullable: false),
                    MuteChannelId = table.Column<decimal>(type: "numeric(20,0)", nullable: false),
                    MutedRoleId = table.Column<decimal>(type: "numeric(20,0)", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_GuildRoleStore", x => x.id);
                });
        }
    }
}
