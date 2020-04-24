using Microsoft.EntityFrameworkCore.Migrations;

namespace RoyalGuard.Migrations
{
    public partial class AddGuilds : Migration
    {
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropColumn(
                name: "DiscordId",
                table: "Warns");

            migrationBuilder.DropColumn(
                name: "DiscordId",
                table: "Mutes");

            migrationBuilder.AddColumn<decimal>(
                name: "GuildId",
                table: "Warns",
                nullable: false,
                defaultValue: 0m);

            migrationBuilder.AddColumn<decimal>(
                name: "UserId",
                table: "Warns",
                nullable: false,
                defaultValue: 0m);

            migrationBuilder.AddColumn<decimal>(
                name: "GuildId",
                table: "Mutes",
                nullable: false,
                defaultValue: 0m);

            migrationBuilder.AddColumn<decimal>(
                name: "UserId",
                table: "Mutes",
                nullable: false,
                defaultValue: 0m);
        }

        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropColumn(
                name: "GuildId",
                table: "Warns");

            migrationBuilder.DropColumn(
                name: "UserId",
                table: "Warns");

            migrationBuilder.DropColumn(
                name: "GuildId",
                table: "Mutes");

            migrationBuilder.DropColumn(
                name: "UserId",
                table: "Mutes");

            migrationBuilder.AddColumn<decimal>(
                name: "DiscordId",
                table: "Warns",
                type: "numeric(20,0)",
                nullable: false,
                defaultValue: 0m);

            migrationBuilder.AddColumn<decimal>(
                name: "DiscordId",
                table: "Mutes",
                type: "numeric(20,0)",
                nullable: false,
                defaultValue: 0m);
        }
    }
}
