import { injectable } from "tsyringe";
import { FollowFetchOpts, IFollowRepository } from "../follow";
import { UserFollow } from "../../domain/model/follow";
import { createDB } from "../../db";
import { userFollows } from "../../mysql_schema";
import { and, desc, eq, lt, lte, sql } from "drizzle-orm";
import { ObjectID } from "../../domain/model/object_id";
import { Clock } from "../../utils/clock";

@injectable()
export class FollowMysqlRepository implements IFollowRepository {
  async save(follow: UserFollow): Promise<void> {
    const db = await createDB();
    const inserted = await db
      .insert(userFollows)
      .values({
        followerId: follow.followerId.id,
        followeeId: follow.followeeId.id,
        followAt: follow.createdAt,
      })
      .onDuplicateKeyUpdate({
        set: {
          id: sql`id`,
        },
      });
    follow.setID(inserted[0].insertId);
  }
  async delete(follow: UserFollow): Promise<void> {
    const db = await createDB();
    if (follow.id !== null) {
      await db.delete(userFollows).where(eq(userFollows.id, follow.id));
    } else {
      await db
        .delete(userFollows)
        .where(
          and(
            eq(userFollows.followerId, follow.followerId.id),
            eq(userFollows.followeeId, follow.followeeId.id)
          )
        );
    }
  }

  async getFollowers(
    userId: ObjectID,
    opts: FollowFetchOpts
  ): Promise<UserFollow[]> {
    const db = await createDB();

    const e1 = eq(userFollows.followeeId, userId.id);
    let cond;
    if (opts.before) {
      cond = and(e1, lte(userFollows.followAt, opts.before));
    } else {
      cond = e1;
    }

    const rows = await db
      .select()
      .from(userFollows)
      .where(cond)
      .orderBy(desc(userFollows.followAt))
      .limit(opts.limit);

    return rows.map((row) => {
      return new UserFollow(
        row.id,
        new ObjectID(row.followerId),
        new ObjectID(row.followeeId),
        new Clock(row.followAt)
      );
    });
  }

  async getFollowings(
    userId: ObjectID,
    opts: FollowFetchOpts
  ): Promise<UserFollow[]> {
    const db = await createDB();

    const e1 = eq(userFollows.followerId, userId.id);
    let cond;
    if (opts.before) {
      cond = and(e1, lte(userFollows.followAt, opts.before));
    } else {
      cond = e1;
    }

    const rows = await db
      .select()
      .from(userFollows)
      .where(cond)
      .orderBy(desc(userFollows.followAt))
      .limit(opts.limit);

    return rows.map((row) => {
      return new UserFollow(
        row.id,
        new ObjectID(row.followerId),
        new ObjectID(row.followeeId),
        new Clock(row.followAt)
      );
    });
  }

  async isFollowing(
    followerId: ObjectID,
    followeeId: ObjectID
  ): Promise<boolean> {
    const db = await createDB();
    const result = await db
      .select()
      .from(userFollows)
      .where(
        and(
          eq(userFollows.followerId, followerId.id),
          eq(userFollows.followeeId, followeeId.id)
        )
      );
    return result.length > 0;
  }
}
