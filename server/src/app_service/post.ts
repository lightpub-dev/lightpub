import { inject, injectable } from "tsyringe";
import { type PostFactory } from "../domain/factory/post";
import { type PostRepository } from "../repository/post";
import { POST_FACTORY, POST_REPOSITORY } from "../registry_key";
import { Post, PostContent } from "../domain/model/post";
import { ObjectID } from "../domain/model/object_id";
import { PostService } from "../domain/service/post";
import { LightpubException } from "../error";
import { clockNow } from "../utils/clock";

export type CreatePostCmd =
  | (
      | {
          // normal post
          content: string;
        }
      | {
          // reply
          content: string;
          replyToId: string;
        }
      | {
          // repost
          repostOfId: string;
        }
      | {
          // quote
          content: string;
          repostOfId: string;
        }
    ) & {
      // common fields
      privacy: "public" | "unlisted" | "follower" | "private";
      authorId: string;
    };

export class InvalidPostError extends LightpubException {
  constructor() {
    super(400, "Invalid post creation request");
  }
}

export class PostNotFoundError extends LightpubException {
  constructor() {
    super(404, "Post not found");
  }
}

@injectable()
export class PostCreateApplicationService {
  constructor(
    @inject(POST_FACTORY) private postFactory: PostFactory,
    @inject(POST_REPOSITORY) private postRepository: PostRepository,
    private postService: PostService
  ) {}

  async createPost(createPostCmd: CreatePostCmd): Promise<{ id: string }> {
    let post: Post;
    if (!("content" in createPostCmd)) {
      // is a repost
      post = await this.postFactory.createRepost(
        new ObjectID(createPostCmd.authorId),
        new ObjectID(createPostCmd.repostOfId),
        createPostCmd.privacy
      );
    } else {
      if ("replyToId" in createPostCmd) {
        // is a reply
        post = await this.postFactory.createReply(
          new ObjectID(createPostCmd.authorId),
          new PostContent(createPostCmd.content),
          new ObjectID(createPostCmd.replyToId),
          createPostCmd.privacy
        );
      } else if ("repostOfId" in createPostCmd) {
        // is a quote
        post = await this.postFactory.createQuote(
          new ObjectID(createPostCmd.authorId),
          new PostContent(createPostCmd.content),
          new ObjectID(createPostCmd.repostOfId),
          createPostCmd.privacy
        );
      } else {
        // is a normal post
        post = await this.postFactory.createPost(
          new ObjectID(createPostCmd.authorId),
          new PostContent(createPostCmd.content),
          createPostCmd.privacy
        );
      }
    }

    // check validity
    const isValid = await this.postService.isValid(post);
    if (!isValid.valid) {
      switch (isValid.reason) {
        case "invalidPostFields":
        case "badRepostPrivacy":
          throw new InvalidPostError();
        case "replyToIdNotFound":
        case "replyToIdNotVisible":
        case "repostOfIdNotFound":
        case "repostOfIdNotVisible":
          throw new PostNotFoundError();
        default:
          throw new Error("Unhandled reason");
      }
    }

    await this.postRepository.save(post);

    return {
      id: post.id.id,
    };
  }

  async deletePost(postId: string, deleterId: string): Promise<void> {
    const post = await this.postRepository.findById(new ObjectID(postId));
    if (post === null) {
      throw new PostNotFoundError();
    }

    if (
      !(await this.postService.isAllowedToDelete(
        new ObjectID(deleterId),
        new ObjectID(postId)
      ))
    ) {
      throw new LightpubException(
        403,
        "You are not allowed to delete this post"
      );
    }

    post.deletedAt = clockNow();
    await this.postRepository.update(post);
  }
}
