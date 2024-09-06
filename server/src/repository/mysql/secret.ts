import { injectable } from "tsyringe";
import { type ISecretRepository } from "../secret";
import { secrets } from "../../mysql_schema";
import { eq } from "drizzle-orm";
import { createDB } from "../../db";

@injectable()
export class SecretMysqlRepository implements ISecretRepository {
  async getSecret(key: string): Promise<string | null> {
    const db = await createDB();
    const result = await db.select().from(secrets).where(eq(secrets.key, key));
    if (result.length === 0) {
      return null;
    }

    return result[0].value;
  }
  async setSecret(key: string, value: string): Promise<void> {
    const db = await createDB();
    await db.delete(secrets).where(eq(secrets.key, key));
    await db.insert(secrets).values({ key, value });
  }
}
