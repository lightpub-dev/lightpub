import { inject, injectable } from "tsyringe";
import { type PostRepository } from "../../repository/post";
import { POST_REPOSITORY } from "../../registry_key";
import { Post } from "../model/post";

@injectable()
export class PostService {
  constructor(
    @inject(POST_REPOSITORY) private postRepository: PostRepository
  ) {}

  async isValid(post: Post): Promise<boolean> {
    if (post.content === null && post.repostOfId === null) {
      // if content is null, the post must be a repost
      return false;
    }

    if (post.replyToId !== null) {
      // if replyToId is set, the replied post must exist
      const replyTo = await this.postRepository.findById(post.replyToId);
      if (replyTo === null) {
        return false;
      }
    }

    if (post.repostOfId !== null) {
      // if repostOfId is set, the reposted post must exist
      const repostOf = await this.postRepository.findById(post.repostOfId);
      if (repostOf === null) {
        return false;
      }
    }

    return true;
  }
}
