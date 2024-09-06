import { inject, injectable } from "tsyringe";
import { type IUserRepository } from "../repository/user";
import { USER_REPOSITORY } from "../registry_key";
import { ObjectID } from "../domain/model/object_id";

export type UserSpec =
  | { username: string; hostname: string | null }
  | { userId: string };

export interface UserDto {
  id: string;
  username: string;
  hostname: string | null;
  nickname: string;
  bio: string;
  createdAt: Date;
}

@injectable()
export class UserApplicationService {
  constructor(
    @inject(USER_REPOSITORY) private userRepository: IUserRepository
  ) {}

  async findUserId(spec: UserSpec): Promise<{
    userId: string;
  } | null> {
    if ("userId" in spec) {
      return { userId: spec.userId };
    }

    const user = await this.userRepository.findByUsernameAndHostname(
      spec.username,
      spec.hostname
    );
    if (user === null) {
      return null;
    }

    return {
      userId: user.id.id,
    };
  }

  async findUserById(id: string): Promise<UserDto | null> {
    const user = await this.userRepository.findById(new ObjectID(id));
    if (user === null) {
      return null;
    }

    return {
      id: user.id.id,
      username: user.username.value,
      hostname: user.hostname,
      nickname: user.nickname.value,
      bio: user.bio,
      createdAt: user.createdAt,
    };
  }
}
