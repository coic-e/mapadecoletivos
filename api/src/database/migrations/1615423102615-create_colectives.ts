import { MigrationInterface, QueryRunner, Table } from "typeorm";

export class createColectives1615423102615 implements MigrationInterface {
  public async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.createTable(
      new Table({
        name: "collectives",
        columns: [
          {
            name: "id",
            type: "integer",
            unsigned: true,
            isPrimary: true,
            isGenerated: true,
            generationStrategy: "increment",
          },
          {
            name: "name",
            type: "varchar",
          },
          {
            name: "latitude",
            type: "decimal",
            scale: 10,
            precision: 2,
          },
          {
            name: "longitude",
            type: "decimal",
            scale: 10,
            precision: 2,
          },
          {
            name: "type",
            type: "varchar",
          },
          {
            name: "city",
            type: "varchar",
          },
          {
            name: "UF",
            type: "varchar",
          },
          {
            name: "email",
            type: "varchar",
          },
          {
            name: "social",
            type: "varchar",
          },
          {
            name: "about",
            type: "varchar",
          },
          {
            name: "created_at",
            type: "timestamp",
            default: "now()",
          },
        ]
      })
    );
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.dropTable("collectives");
  }
  
}
