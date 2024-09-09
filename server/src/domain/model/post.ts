import { Clock } from "../../utils/clock";
import { ValueObject } from "../../utils/eq";
import { ObjectID } from "./object_id";

export class Post {
  constructor(
    public id: ObjectID,
    public url: string | null,
    public authorId: ObjectID,
    public content: PostContent | null,
    public privacy: "public" | "unlisted" | "follower" | "private",
    public replyToId: ObjectID | null,
    public repostOfId: ObjectID | null,
    public createdAt: Clock,
    public deletedAt: Clock | null
  ) {}
}

export class PostContent implements ValueObject {
  constructor(private content: string) {
    if (content.length > 2048) {
      throw new Error("Content too long");
    }
  }

  equals(other: ValueObject): boolean {
    if (other instanceof PostContent) {
      return this.content === other.content;
    }
    return false;
  }

  toString(): string {
    return this.content;
  }
}
