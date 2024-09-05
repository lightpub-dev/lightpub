import { Service } from "typedi";
import { Nickname, User, Username } from "../model/user";
import { IIDGenerator } from "./object_id";
import { clockNow, now } from "../../utils/clock";

export interface IUserFactory {
  create: (
    username: string,
    password: string,
    nickname: string
  ) => Promise<User>;
}

@Service()
export class DefaultUserFactory implements IUserFactory {
  constructor(private idGenerator: IIDGenerator) {}

  async create(
    username: string,
    password: string,
    nickname: string
  ): Promise<User> {
    const hash = await Bun.password.hash(password, {
      algorithm: "bcrypt",
    });

    return new User(
      this.idGenerator.generate(),
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
