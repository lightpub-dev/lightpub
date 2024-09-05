import { and, eq, isNull } from "drizzle-orm";
import { db } from "../../db";
import { ObjectID } from "../../domain/model/object_id";
import { Nickname, User, Username } from "../../domain/model/user";
import { users } from "../../sqlite_schema";
import { IUserRepository } from "../user";
import { Clock } from "../../utils/clock";

export class UserSqliteRepository implements IUserRepository {
  async save(user: User): Promise<void> {
    db.insert(users).values({
      id: user.id.id,
      username: user.username.value,
      hostname: user.hostname,
      nickname: user.nickname.value,
      password: user.password,
      bio: user.bio,
      url: user.url,
      privateKey: user.privateKey,
      publicKey: user.publicKey,
      createdAt: user.createdAt.asNumber(),
      deletedAt: user.deletedAt?.asNumber(),
    });
  }

  async findById(id: ObjectID): Promise<User | null> {
    const result = await db.select().from(users).where(eq(users.id, id.id));

    if (result.length === 0) {
      return null;
    }

    if (result.length > 1) {
      throw new Error("Multiple users found");
    }

    return this.buildUser(result[0]);
  }

  async findByUsernameAndHostname(
    username: string,
    hostname: string | null
  ): Promise<User | null> {
    let eqClause = eq(users.username, username);
    if (hostname !== null) {
      eqClause = and(eqClause, eq(users.hostname, hostname))!;
    } else {
      eqClause = and(eqClause, isNull(users.hostname))!;
    }
    const result = await db.select().from(users).where(eqClause);

    if (result.length === 0) {
      return null;
    }

    if (result.length > 1) {
      throw new Error("Multiple users found");
    }

    return this.buildUser(result[0]);
  }

  private buildUser(u: {
    id: string;
    username: string;
    hostname: string | null;
    password: string | null;
    nickname: string;
    bio: string;
    url: string | null;
    privateKey: string | null;
    publicKey: string | null;
    createdAt: number;
    deletedAt: number | null;
  }): User {
    return new User(
      new ObjectID(u.id),
      new Username(u.username),
      u.hostname,
      u.password,
      new Nickname(u.nickname),
      u.bio,
      u.url,
      u.privateKey,
      u.publicKey,
      new Clock(u.createdAt),
      u.deletedAt ? new Clock(u.deletedAt) : null
    );
  }
}
