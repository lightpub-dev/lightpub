import { inject, injectable } from "tsyringe";
import { type IUserRepository } from "../../repository/user";
import { User } from "../model/user";
import { USER_REPOSITORY } from "../../registry_key";

@injectable()
export class UserService {
  constructor(
    @inject(USER_REPOSITORY) private userRepository: IUserRepository
  ) {}

  async isUnique(user: User): Promise<boolean> {
    const idConflict = await this.userRepository.findById(user.id);
    if (idConflict !== null) {
      return false;
    }

    const usernameConflict =
      await this.userRepository.findByUsernameAndHostname(
        user.username.value,
        user.hostname
      );
    if (usernameConflict !== null) {
      return false;
    }

    return true;
  }
}
