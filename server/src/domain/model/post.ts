import { Clock } from "../../utils/clock";
import { ObjectID } from "./object_id";

export class Post {
  constructor(
    public id: ObjectID,
    public url: string,
    public authorId: string,
    public content: string | null,
    public privacy: "public" | "unlisted" | "follower" | "private",
    public replyToId: string | null,
    public repostOfId: string | null,
    public counter: {
      reply: number;
      repost: number;
      favorite: number;
    },
    public reactions: { name: string; count: number }[],
    public createdAt: Clock,
    public deletedAt: Clock | null
  ) {}
}
