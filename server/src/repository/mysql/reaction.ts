import { injectable } from "tsyringe";
import { type ReactionRepository } from "../reaction";
import {
  PostFavorite,
  PostBookmark,
  PostReaction,
} from "../../domain/model/reaction";
import { createDB } from "../../db";
import {
  postBookmarks,
  postFavorites,
  postReactions,
} from "../../mysql_schema";
import { and, eq, sql } from "drizzle-orm";
import { ObjectID } from "../../domain/model/object_id";

@injectable()
export class ReactionMysqlRepository implements ReactionRepository {
  async save(
    reaction: PostFavorite | PostBookmark | PostReaction
  ): Promise<void> {
    const db = await createDB();
    if (reaction instanceof PostFavorite) {
      const result = await db.insert(postFavorites).values({
        postId: reaction.postId.id,
        userId: reaction.userId.id,
        favoritedAt: reaction.favoritedAt,
      });
      reaction.id = result[0].insertId;
    } else if (reaction instanceof PostBookmark) {
      const result = await db.insert(postBookmarks).values({
        postId: reaction.postId.id,
        userId: reaction.userId.id,
        bookmarkedAt: reaction.bookmarkedAt,
      });
      reaction.id = result[0].insertId;
    } else if (reaction instanceof PostReaction) {
      const result = await db.insert(postReactions).values({
        postId: reaction.postId.id,
        userId: reaction.userId.id,
        reaction: reaction.reaction,
        reactedAt: reaction.reactedAt,
      });
      reaction.id = result[0].insertId;
    } else {
      throw new Error("Unknown reaction type");
    }
  }

  async delete(
    reaction: PostFavorite | PostBookmark | PostReaction
  ): Promise<void> {
    const reactionId = reaction.id;
    if (reactionId === undefined) {
      throw new Error("Reaction id is not set");
    }

    const db = await createDB();
    if (reaction instanceof PostFavorite) {
      await db.delete(postFavorites).where(eq(postFavorites.id, reactionId));
    } else if (reaction instanceof PostBookmark) {
      await db.delete(postBookmarks).where(eq(postBookmarks.id, reactionId));
    } else if (reaction instanceof PostReaction) {
      await db.delete(postReactions).where(eq(postReactions.id, reactionId));
    } else {
      throw new Error("Unknown reaction type");
    }
  }

  async find(
    reactionType: "favorite",
    userId: ObjectID,
    postId: ObjectID
  ): Promise<PostFavorite | null>;
  async find(
    reactionType: "bookmark",
    userId: ObjectID,
    postId: ObjectID
  ): Promise<PostBookmark | null>;
  async find(
    reactionType: "reaction",
    userId: ObjectID,
    postId: ObjectID
  ): Promise<PostReaction[]>;
  find(
    reactionType: "reaction",
    userId: ObjectID,
    postId: ObjectID,
    emoji: string
  ): Promise<PostReaction | null>;
  async find(
    reactionType: "favorite" | "bookmark" | "reaction",
    userId: ObjectID,
    postId: ObjectID,
    reaction?: string
  ): Promise<any> {
    const db = await createDB();
    if (reactionType === "favorite") {
      const result = await db
        .select()
        .from(postFavorites)
        .where(
          and(
            eq(postFavorites.userId, userId.id),
            eq(postFavorites.postId, postId.id)
          )
        );
      if (result.length === 0) return null;
      return this.buildPostFavorite(result[0]);
    } else if (reactionType === "bookmark") {
      const result = await db
        .select()
        .from(postBookmarks)
        .where(
          and(
            eq(postBookmarks.userId, userId.id),
            eq(postBookmarks.postId, postId.id)
          )
        );
      if (result.length === 0) return null;
      return this.buildPostBookmark(result[0]);
    } else if (reactionType === "reaction") {
      const result = await db
        .select()
        .from(postReactions)
        .where(
          and(
            eq(postReactions.userId, userId.id),
            eq(postReactions.postId, postId.id),
            reaction === undefined
              ? sql`1`
              : eq(postReactions.reaction, reaction)
          )
        );
      if (reaction === undefined) {
        return result.map(this.buildPostReaction);
      } else {
        if (result.length === 0) return null;
        return this.buildPostReaction(result[0]);
      }
    } else {
      throw new Error("Unknown reaction type");
    }
  }

  private buildPostFavorite(result: {
    id: number;
    postId: string;
    userId: string;
    favoritedAt: Date;
  }): PostFavorite {
    return new PostFavorite(
      new ObjectID(result.postId),
      new ObjectID(result.userId),
      result.favoritedAt,
      result.id
    );
  }

  private buildPostBookmark(result: {
    id: number;
    postId: string;
    userId: string;
    bookmarkedAt: Date;
  }): PostBookmark {
    return new PostBookmark(
      new ObjectID(result.postId),
      new ObjectID(result.userId),
      result.bookmarkedAt,
      result.id
    );
  }

  private buildPostReaction(result: {
    id: number;
    postId: string;
    userId: string;
    reaction: string;
    reactedAt: Date;
  }): PostReaction {
    return new PostReaction(
      new ObjectID(result.postId),
      new ObjectID(result.userId),
      result.reaction,
      result.reactedAt,
      result.id
    );
  }
}
