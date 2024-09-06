import { ObjectID } from "../model/object_id";
import { Post } from "../model/post";

export interface PostFactory {
  createPost(authorId: ObjectID, content: string): Promise<Post>;
  createReply(
    authorId: ObjectID,
    content: string,
    replyToId: ObjectID
  ): Promise<Post>;
  createRepost(authorId: ObjectID, repostOfId: ObjectID): Promise<Post>;
  createQuote(
    authorId: ObjectID,
    content: string,
    repostOfId: ObjectID
  ): Promise<Post>;
}

export class DefaultPostFactory implements PostFactory {
  createPost(authorId: ObjectID, content: string): Promise<Post> {}

  createReply(
    authorId: ObjectID,
    content: string,
    replyToId: ObjectID
  ): Promise<Post> {
    throw new Error("Method not implemented.");
  }

  createRepost(authorId: ObjectID, repostOfId: ObjectID): Promise<Post> {
    throw new Error("Method not implemented.");
  }

  createQuote(
    authorId: ObjectID,
    content: string,
    repostOfId: ObjectID
  ): Promise<Post> {
    throw new Error("Method not implemented.");
  }
}
