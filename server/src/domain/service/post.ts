import { inject, injectable } from "tsyringe";
import { type PostRepository } from "../../repository/post";
import { FOLLOW_REPOSITORY, POST_REPOSITORY } from "../../registry_key";
import { Post } from "../model/post";
import { ObjectID } from "../model/object_id";
import { type IFollowRepository } from "../../repository/follow";
import { LightpubInternalException } from "../../error";

export class PostNotFoundException extends LightpubInternalException {}

export type ValidResult =
  | {
      valid: true;
    }
  | {
      valid: false;
      reason:
        | "replyToIdNotFound"
        | "replyToIdNotVisible"
        | "repostOfIdNotFound"
        | "repostOfIdNotVisible"
        | "invalidPostFields"
        | "badRepostPrivacy";
    };

@injectable()
export class PostService {
  constructor(
    @inject(POST_REPOSITORY) private postRepository: PostRepository,
    @inject(FOLLOW_REPOSITORY) private followRepository: IFollowRepository
  ) {}

  async isValid(post: Post): Promise<ValidResult> {
    if (post.content === null && post.repostOfId === null) {
      // if content is null, the post must be a repost
      return {
        valid: false,
        reason: "invalidPostFields",
      };
    }

    if (post.replyToId !== null) {
      // if replyToId is set, the replied post must exist and visible to the author
      const replyTo = await this.postRepository.findById(post.replyToId);
      if (replyTo === null) {
        return {
          valid: false,
          reason: "replyToIdNotFound",
        };
      }
      if (!(await this.isVisibleTo(post.authorId, post.replyToId))) {
        return {
          valid: false,
          reason: "replyToIdNotVisible",
        };
      }
    }

    if (post.repostOfId !== null) {
      // if repostOfId is set, the reposted post must exist and visible to the author
      const repostOf = await this.postRepository.findById(post.repostOfId);
      if (repostOf === null) {
        return {
          valid: false,
          reason: "repostOfIdNotFound",
        };
      }
      if (!(await this.isVisibleTo(post.authorId, post.repostOfId))) {
        return {
          valid: false,
          reason: "repostOfIdNotVisible",
        };
      }

      if (post.content === null) {
        // when repost (not quote)
        // it is impossible to repost a follower-only or private post
        if (repostOf.privacy === "follower" || repostOf.privacy === "private") {
          return {
            valid: false,
            reason: "badRepostPrivacy",
          };
        }
        // it is also impossible to repost with follower-only or private privacy
        if (post.privacy === "follower" || post.privacy === "private") {
          return {
            valid: false,
            reason: "badRepostPrivacy",
          };
        }
      }
    }

    return {
      valid: true,
    };
  }

  async isVisibleTo(viewerId: ObjectID, postId: ObjectID): Promise<boolean> {
    const post = await this.postRepository.findById(postId);
    if (post === null) {
      throw new PostNotFoundException();
    }
    if (["public", "unlisted"].includes(post.privacy)) {
      // public or unlisted posts are always visible to everyone
      return true;
    }

    if (post.privacy === "follower") {
      // follower posts are visible to followers
      return this.followRepository.isFollowing(viewerId, post.authorId);
    }

    if (post.privacy === "private") {
      // private posts are visible only to the author
      return viewerId.equals(post.authorId);
    }

    // unreachable
    console.error("unreachable");
    throw new LightpubInternalException();
  }

  async isAllowedToDelete(
    viewerId: ObjectID,
    postId: ObjectID
  ): Promise<boolean> {
    const post = await this.postRepository.findById(postId);
    if (post === null) {
      throw new PostNotFoundException();
    }

    return viewerId.equals(post.authorId);
  }
}
