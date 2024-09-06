import { injectable } from "tsyringe";
import { Clock, clockNow } from "../../utils/clock";
import { UserFollow } from "../model/follow";
import { ObjectID } from "../model/object_id";
import { UserFollowService } from "../service/follow";
import { LightpubException } from "../../error";

export interface FollowFactory {
  createFollow(fromId: ObjectID, toId: ObjectID): Promise<UserFollow>;
}

class InvalidFollowException extends LightpubException {
  constructor() {
    super(400, "follower or followee does not exist");
  }
}

@injectable()
export class DefaultFollowFactory implements FollowFactory {
  constructor(private followService: UserFollowService) {}

  async createFollow(fromId: ObjectID, toId: ObjectID): Promise<UserFollow> {
    const follow = new UserFollow(null, fromId, toId, clockNow());
    if (!(await this.followService.isValid(follow))) {
      throw new InvalidFollowException();
    }

    return follow;
  }
}
