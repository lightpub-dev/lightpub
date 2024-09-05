import { Nickname, User, Username } from "../model/user";
import { type IIDGenerator } from "./object_id";
import { clockNow, now } from "../../utils/clock";
import { inject, injectable } from "tsyringe";
import { ID_GENERATOR } from "../../registry_key";

export interface IUserFactory {
  create: (
    username: string,
    password: string,
    nickname: string
  ) => Promise<User>;
}

@injectable()
export class DefaultUserFactory implements IUserFactory {
  constructor(@inject(ID_GENERATOR) private idGenerator: IIDGenerator) {}

  async create(
    username: string,
    password: string,
    nickname: string
  ): Promise<User> {
    const hash = await Bun.password.hash(password, {
      algorithm: "bcrypt",
    });

    return new User(
      this.idGenerator.generateRandom(),
      new Username(username),
      null,
      hash,
      new Nickname(nickname),
      "",
      null,
      null,
      null,
      clockNow(),
      null
    );
  }
}
