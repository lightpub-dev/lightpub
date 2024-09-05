import { injectable } from "tsyringe";
import { type ISecretRepository } from "../secret";
import { db } from "../../db";
import { secrets } from "../../sqlite_schema";
import { eq } from "drizzle-orm";

@injectable()
export class SecretSqliteRepository implements ISecretRepository {
  async getSecret(key: string): Promise<string | null> {
    const result = await db.select().from(secrets).where(eq(secrets.key, key));
    if (result.length === 0) {
      return null;
    }

    return result[0].value;
  }
  async setSecret(key: string, value: string): Promise<void> {
    await db.delete(secrets).where(eq(secrets.key, key));
    await db.insert(secrets).values({ key, value });
  }
}
