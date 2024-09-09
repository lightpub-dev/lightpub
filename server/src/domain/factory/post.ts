import { inject, injectable } from "tsyringe";
import { ObjectID } from "../model/object_id";
import { Post, PostContent } from "../model/post";
import { type IIDGenerator } from "./object_id";
import { ID_GENERATOR } from "../../registry_key";
import { clockNow } from "../../utils/clock";

export type PostPrivacy = "public" | "unlisted" | "follower" | "private";

export interface PostFactory {
  createPost(
    authorId: ObjectID,
    content: PostContent,
    privacy: PostPrivacy
  ): Promise<Post>;
  createReply(
    authorId: ObjectID,
    content: PostContent,
    replyToId: ObjectID,

    privacy: PostPrivacy
  ): Promise<Post>;
  createRepost(
    authorId: ObjectID,
    repostOfId: ObjectID,
    privacy: PostPrivacy
  ): Promise<Post>;
  createQuote(
    authorId: ObjectID,
    content: PostContent,
    repostOfId: ObjectID,
    privacy: PostPrivacy
  ): Promise<Post>;
}

@injectable()
export class DefaultPostFactory implements PostFactory {
  constructor(@inject(ID_GENERATOR) private idGen: IIDGenerator) {}

  async createPost(
    authorId: ObjectID,
    content: PostContent,
    privacy: PostPrivacy
  ): Promise<Post> {
    return new Post(
      this.idGen.generate(),
      null,
      authorId,
      content,
      privacy,
      null,
      null,
      clockNow(),
      null
    );
  }

  async createReply(
    authorId: ObjectID,
    content: PostContent,
    replyToId: ObjectID,
    privacy: PostPrivacy
  ): Promise<Post> {
    return new Post(
      this.idGen.generate(),
      null,
      authorId,
      content,
      privacy,
      replyToId,
      null,
      clockNow(),
      null
    );
  }

  async createRepost(
    authorId: ObjectID,
    repostOfId: ObjectID,
    privacy: PostPrivacy
  ): Promise<Post> {
    return new Post(
      this.idGen.generate(),
      null,
      authorId,
      null,
      privacy,
      null,
      repostOfId,
      clockNow(),
      null
    );
  }

  async createQuote(
    authorId: ObjectID,
    content: PostContent,
    repostOfId: ObjectID,
    privacy: PostPrivacy
  ): Promise<Post> {
    return new Post(
      this.idGen.generate(),
      null,
      authorId,
      content,
      privacy,
      null,
      repostOfId,
      clockNow(),
      null
    );
  }
}
