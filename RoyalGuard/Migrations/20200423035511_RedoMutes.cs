using Microsoft.EntityFrameworkCore.Migrations;

namespace RoyalGuard.Migrations
{
    public partial class RedoMutes : Migration
    {
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropColumn(
                name: "MutedRoleId",
                table: "Mutes");
        }

        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AddColumn<decimal>(
                name: "MutedRoleId",
                table: "Mutes",
                type: "numeric(20,0)",
                nullable: false,
                defaultValue: 0m);
        }
    }
}
