import { inject, injectable } from "tsyringe";
import { type ReactionRepository } from "../../repository/reaction";
import { PostService } from "./post";
import { UserService } from "./user";
import { PostBookmark, PostFavorite, PostReaction } from "../model/reaction";
import { REACTION_REPOSITORY } from "../../registry_key";

type ValidResult =
  | {
      valid: true;
    }
  | {
      valid: false;
      reason: "postNotFound" | "userNotFound" | "invalidReaction";
    };

@injectable()
export class PostReactionService {
  constructor(
    @inject(REACTION_REPOSITORY)
    private postReactionRepository: ReactionRepository,
    private postService: PostService,
    private userService: UserService
  ) {}

  async isValid(fav: PostFavorite): Promise<ValidResult>;
  async isValid(bm: PostBookmark): Promise<ValidResult>;
  async isValid(r: PostReaction): Promise<ValidResult>;
  async isValid(
    r: PostFavorite | PostBookmark | PostReaction
  ): Promise<ValidResult> {
    if (!(await this.userService.exists(r.userId))) {
      return {
        valid: false,
        reason: "userNotFound",
      };
    }

    if (!(await this.postService.isVisibleTo(r.userId, r.postId))) {
      return {
        valid: false,
        reason: "postNotFound",
      };
    }

    return { valid: true };
  }
}
