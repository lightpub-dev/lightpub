import { inject, injectable } from "tsyringe";
import { type PostFactory } from "../domain/factory/post";
import { type PostRepository } from "../repository/post";
import {
  POST_FACTORY,
  POST_REPOSITORY,
  REACTION_REPOSITORY,
} from "../registry_key";
import { Post, PostContent } from "../domain/model/post";
import { ObjectID } from "../domain/model/object_id";
import { PostService } from "../domain/service/post";
import { LightpubException } from "../error";
import { clockNow } from "../utils/clock";
import { type ReactionRepository } from "../repository/reaction";
import { PostReactionService } from "../domain/service/reaction";
import {
  PostBookmark,
  PostFavorite,
  PostReaction,
} from "../domain/model/reaction";

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

export interface PostDto {
  id: string;
  url: string | null;
  authorId: string;
  createdAt: Date;
  content: string | null;
  privacy: "public" | "unlisted" | "follower" | "private";
  replyToId: string | null;
  repostOfId: string | null;
}

@injectable()
export class PostFetchApplicationService {
  constructor(
    @inject(POST_REPOSITORY) private postRepository: PostRepository,
    private postService: PostService
  ) {}

  async fetchPost(
    postId: string,
    viewerId: string | null
  ): Promise<PostDto | null> {
    const post = await this.postRepository.findById(new ObjectID(postId));
    if (post === null) {
      return null;
    }

    // check visibility
    if (
      !(await this.postService.isVisibleTo(
        viewerId !== null ? new ObjectID(viewerId) : null,
        post.id
      ))
    ) {
      return null;
    }

    return {
      id: post.id.id,
      url: post.url,
      authorId: post.authorId.id,
      createdAt: post.createdAt,
      content: post.content?.toString() ?? null,
      privacy: post.privacy,
      replyToId: post.replyToId?.id ?? null,
      repostOfId: post.repostOfId?.id ?? null,
    };
  }
}

class ReactionUserNotFoundException extends LightpubException {
  constructor() {
    super(404, "User not found");
  }
}

class ReactionPostNotFoundException extends LightpubException {
  constructor() {
    super(404, "Post not found");
  }
}

@injectable()
export class PostReactionApplicationService {
  constructor(
    @inject(REACTION_REPOSITORY) private reactionRepository: ReactionRepository,
    private reactionService: PostReactionService
  ) {}

  async favoritePost(userId: string, postId: string): Promise<void> {
    const fav = new PostFavorite(
      new ObjectID(postId),
      new ObjectID(userId),
      clockNow()
    );

    await this.store(fav);
  }

  async deleteFavorite(userId: string, postId: string): Promise<void> {
    const fav = await this.reactionRepository.find(
      "favorite",
      new ObjectID(userId),
      new ObjectID(postId)
    );
    if (fav === null) return;
    await this.reactionRepository.delete(fav);
  }

  async bookmarkPost(userId: string, postId: string): Promise<void> {
    const bookmark = new PostBookmark(
      new ObjectID(postId),
      new ObjectID(userId),
      clockNow()
    );

    await this.store(bookmark);
  }

  async deleteBookmark(userId: string, postId: string): Promise<void> {
    const book = await this.reactionRepository.find(
      "bookmark",
      new ObjectID(userId),
      new ObjectID(postId)
    );
    if (book === null) return;
    await this.reactionRepository.delete(book);
  }

  async reactionPost(
    userId: string,
    postId: string,
    emoji: string
  ): Promise<void> {
    const reaction = new PostReaction(
      new ObjectID(postId),
      new ObjectID(userId),
      emoji,
      clockNow()
    );

    await this.store(reaction);
  }

  async deleteReaction(
    userId: string,
    postId: string,
    emoji: string
  ): Promise<void> {
    const reaction = await this.reactionRepository.find(
      "reaction",
      new ObjectID(userId),
      new ObjectID(postId),
      emoji
    );
    if (reaction === null) return;
    await this.reactionRepository.delete(reaction);
  }

  private async store(
    r: PostFavorite | PostBookmark | PostReaction
  ): Promise<void> {
    const valid = await this.reactionService.isValid(r as any);
    if (!valid.valid) {
      switch (valid.reason) {
        case "userNotFound":
          throw new ReactionUserNotFoundException();
        case "postNotFound":
          throw new ReactionPostNotFoundException();
        default:
          throw new Error("Unhandled reason");
      }
    }

    await this.reactionRepository.save(r);
  }
}
