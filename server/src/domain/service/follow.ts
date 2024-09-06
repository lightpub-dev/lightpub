import { inject, injectable } from "tsyringe";
import { UserService } from "./user";
import { UserFollow } from "../model/follow";
import { USER_FACTORY } from "../../registry_key";
import { type IUserFactory } from "../factory/user";

@injectable()
export class UserFollowService {
  constructor(
    private userService: UserService,
    @inject(USER_FACTORY) private userFactory: IUserFactory
  ) {}

  async isValid(follow: UserFollow): Promise<boolean> {
    // check existence of from user and to user
    if (!(await this.userService.exists(follow.followerId))) {
      return false;
    }
    if (!(await this.userService.exists(follow.followeeId))) {
      return false;
    }

    return true;
  }
}
