using System;
using Microsoft.EntityFrameworkCore.Migrations;

namespace RoyalGuard.Migrations
{
    public partial class guildrolestorage : Migration
    {
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AddColumn<decimal>(
                name: "MutedRoleId",
                table: "Mutes",
                nullable: false,
                defaultValue: 0m);

            migrationBuilder.CreateTable(
                name: "GuildRoleStore",
                columns: table => new
                {
                    id = table.Column<Guid>(nullable: false),
                    GuildId = table.Column<decimal>(nullable: false),
                    MutedRoleId = table.Column<decimal>(nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_GuildRoleStore", x => x.id);
                });
        }

        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "GuildRoleStore");

            migrationBuilder.DropColumn(
                name: "MutedRoleId",
                table: "Mutes");
        }
    }
}
