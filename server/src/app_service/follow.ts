import { inject, injectable } from "tsyringe";
import { type FollowFactory } from "../domain/factory/follow";
import {
  FOLLOW_FACTORY,
  FOLLOW_REPOSITORY,
  USER_REPOSITORY,
} from "../registry_key";
import { type IFollowRepository } from "../repository/follow";
import { ObjectID } from "../domain/model/object_id";
import { UserDto } from "./user";
import { type IUserRepository } from "../repository/user";

export interface GetFollowOpts {
  limit: number;
  before?: Date;
}

export interface FollowUserDto extends UserDto {
  followAt: Date;
}

@injectable()
export class FollowApplicationService {
  constructor(
    @inject(FOLLOW_FACTORY) private followFactory: FollowFactory,
    @inject(FOLLOW_REPOSITORY) private followRepository: IFollowRepository,
    @inject(USER_REPOSITORY) private userRepository: IUserRepository
  ) {}

  async follow(followerId: string, followeeId: string): Promise<void> {
    const follow = await this.followFactory.createFollow(
      new ObjectID(followerId),
      new ObjectID(followeeId)
    );
    await this.followRepository.save(follow);
  }

  async unfollow(followerId: string, followeeId: string): Promise<void> {
    const follow = await this.followFactory.createFollow(
      new ObjectID(followerId),
      new ObjectID(followeeId)
    );
    await this.followRepository.delete(follow);
  }

  async getFollowings(
    userId: string,
    opts: GetFollowOpts
  ): Promise<FollowUserDto[]> {
    // TODO: join N+1
    const followings = await this.followRepository.getFollowings(
      new ObjectID(userId),
      { limit: opts.limit, before: opts.before }
    );
    const users = await Promise.all(
      followings.map(async (f) => {
        const user = await this.userRepository.findById(f.followeeId);
        if (user === null) {
          throw new Error("non-existent user found in followings");
        }
        return {
          id: user.id.id,
          username: user.username.value,
          hostname: user.hostname,
          nickname: user.nickname.value,
          bio: user.bio,
          createdAt: user.createdAt,
          followAt: f.createdAt,
        };
      })
    );
    return users;
  }

  async getFollowers(
    userId: string,
    opts: GetFollowOpts
  ): Promise<FollowUserDto[]> {
    // TODO: join N+1
    const followers = await this.followRepository.getFollowers(
      new ObjectID(userId),
      { limit: opts.limit, before: opts.before }
    );
    const users = await Promise.all(
      followers.map(async (f) => {
        const user = await this.userRepository.findById(f.followerId);
        if (user === null) {
          throw new Error("non-existent user found in followers");
        }
        return {
          id: user.id.id,
          username: user.username.value,
          hostname: user.hostname,
          nickname: user.nickname.value,
          bio: user.bio,
          createdAt: user.createdAt,
          followAt: f.createdAt,
        };
      })
    );
    return users;
  }
}
