import { type IUserRepository } from "../repository/user";
import { UserService } from "../domain/service/user";
import { User } from "../domain/model/user";
import { type IUserFactory } from "../domain/factory/user";
import {
  JsonWebTokenError,
  NotBeforeError,
  sign,
  TokenExpiredError,
  verify,
} from "jsonwebtoken";
import { inject, injectable } from "tsyringe";
import { USER_FACTORY, USER_REPOSITORY } from "../registry_key";
import { JWTSecretProvider } from "./jwt_secret";

@injectable()
export class AuthApplicationService {
  constructor(
    @inject(USER_REPOSITORY) private userRepository: IUserRepository,
    private userService: UserService,
    @inject(USER_FACTORY) private userFactory: IUserFactory,
    private jwtSecretProvider: JWTSecretProvider
  ) {}

  async register(params: {
    username: string;
    nickname: string;
    password: string;
  }): Promise<{
    userId: string;
  }> {
    // Create a new user
    const newUser = await this.userFactory.create(
      params.username,
      params.password,
      params.nickname
    );

    // check for conflicts
    if (!(await this.userService.isUnique(newUser))) {
      throw new Error("User already exists");
    }

    await this.userRepository.save(newUser);

    return { userId: newUser.id.id };
  }

  async login(params: { username: string; password: string }): Promise<null | {
    token: string;
  }> {
    const privateKey = await this.jwtSecretProvider.privateKey();

    // get user
    const user = await this.userRepository.findByUsernameAndHostname(
      params.username,
      null
    );
    if (user === null || user.password === null) {
      return null;
    }

    // bcrypt verify
    const ok = await Bun.password.verify(
      params.password,
      user.password,
      "bcrypt"
    );
    if (!ok) {
      return null;
    }

    // generate token
    const iat = Math.floor(Date.now() / 1000);
    const token = sign(
      {
        sub: user.id.id,
        iat: iat,
      },
      privateKey,
      {
        algorithm: "RS256",
      }
    );
    return { token };
  }

  async verifyToken(token: string): Promise<
    | { success: true; userId: string }
    | {
        success: false;
        message: string;
      }
  > {
    const publicKey = await this.jwtSecretProvider.publicKey();
    try {
      const decoded = verify(token, publicKey, {
        algorithms: ["RS256"],
      }) as { sub: string; iat: number };

      return { success: true, userId: decoded.sub };
    } catch (ex) {
      if (ex instanceof TokenExpiredError) {
        return { success: false, message: "Token expired" };
      } else if (ex instanceof JsonWebTokenError) {
        return { success: false, message: "Invalid token" };
      } else if (ex instanceof NotBeforeError) {
        return { success: false, message: "Token is not active yet" };
      }
      throw ex;
    }
  }
}
