import { UserFollow } from "../domain/model/follow";
import { ObjectID } from "../domain/model/object_id";

export interface FollowFetchOpts {
  limit: number;
  before?: Date;
}

export interface IFollowRepository {
  save(follow: UserFollow): Promise<void>;
  delete(follow: UserFollow): Promise<void>;
  getFollowers(userId: ObjectID, opts: FollowFetchOpts): Promise<UserFollow[]>;
  getFollowings(userId: ObjectID, opts: FollowFetchOpts): Promise<UserFollow[]>;
  isFollowing(followerId: ObjectID, followeeId: ObjectID): Promise<boolean>;
}
