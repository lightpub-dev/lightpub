import { inject, injectable } from "tsyringe";
import { type IUserRepository } from "../../repository/user";
import { User } from "../model/user";
import { USER_REPOSITORY } from "../../registry_key";
import { ObjectID } from "../model/object_id";

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

  async exists(userId: ObjectID): Promise<boolean> {
    return (await this.userRepository.findById(userId)) !== null;
  }

  isGoodPassword(password: string): boolean {
    // password must:
    // - be at least 8 characters long and at most 48 characters long
    // - contain at least one uppercase letter
    // - contain at least one lowercase letter
    // - contain at least one digit
    // - contain at least one special character (!?@#$%^&*=+-_)

    if (password.length < 8 || password.length > 48) {
      return false;
    }

    let hasUpper = false;
    let hasLower = false;
    let hasDigit = false;
    let hasSpecial = false;
    const specialCharacters = "!?@#$%^&*=+-_";

    for (const ch of password) {
      if (/[A-Z]/.test(ch)) {
        hasUpper = true;
      } else if (/[a-z]/.test(ch)) {
        hasLower = true;
      } else if (/[0-9]/.test(ch)) {
        hasDigit = true;
      } else if (specialCharacters.includes(ch)) {
        hasSpecial = true;
      } else {
        // invalid character
        return false;
      }
    }

    return hasUpper && hasLower && hasDigit && hasSpecial;
  }
}
