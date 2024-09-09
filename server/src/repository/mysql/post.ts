import { injectable } from "tsyringe";
import { type PostRepository } from "../post";
import { ObjectID } from "../../domain/model/object_id";
import { Post, PostContent } from "../../domain/model/post";
import { createDB } from "../../db";
import { posts } from "../../mysql_schema";
import { eq } from "drizzle-orm";
import { Clock } from "../../utils/clock";

@injectable()
export class PostMysqlRepository implements PostRepository {
  async findById(id: ObjectID): Promise<Post | null> {
    const db = await createDB();
    const result = await db.select().from(posts).where(eq(posts.id, id.id));
    if (result.length === 0) return null;
    if (result.length > 1) throw new Error("Multiple posts found");
    return this.buildPost(result[0]);
  }

  async save(post: Post): Promise<void> {
    const db = await createDB();
    await db.insert(posts).values({
      id: post.id.id,
      url: post.url,
      authorId: post.authorId.id,
      content: post.content?.toString(),
      privacy: post.privacy,
      replyToId: post.replyToId?.id,
      repostOfId: post.repostOfId?.id,
      createdAt: post.createdAt,
      deletedAt: post.deletedAt,
    });
  }

  private buildPost(result: {
    id: string;
    url: string | null;
    authorId: string;
    createdAt: Date;
    deletedAt: Date | null;
    content: string | null;
    privacy: "public" | "unlisted" | "follower" | "private";
    replyToId: string | null;
    repostOfId: string | null;
  }): Post {
    return new Post(
      new ObjectID(result.id),
      result.url,
      new ObjectID(result.authorId),
      result.content !== null ? new PostContent(result.content) : null,
      result.privacy,
      result.replyToId !== null ? new ObjectID(result.replyToId) : null,
      result.repostOfId !== null ? new ObjectID(result.repostOfId) : null,
      new Clock(result.createdAt),
      result.deletedAt !== null ? new Clock(result.deletedAt) : null
    );
  }
}
