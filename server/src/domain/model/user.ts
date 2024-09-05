import { Clock } from "../../utils/clock";
import { ValueObject } from "../../utils/eq";
import { ObjectID } from "./object_id";

export class Username implements ValueObject {
  constructor(public value: string) {
    if (value.length < 3 || value.length > 32) {
      throw new Error("Invalid username");
    }
    for (let ch of value) {
      if (!/^[a-zA-Z0-9_\-]+$/.test(ch)) {
        throw new Error("Invalid username");
      }
    }
    if (
      value.includes("--") ||
      value.includes("__") ||
      value.startsWith("-") ||
      value.startsWith("_")
    ) {
      throw new Error("Invalid username");
    }
  }

  equals(other: ValueObject): boolean {
    if (other instanceof Username) {
      return this.value === other.value;
    }
    return false;
  }
}

export class Nickname implements ValueObject {
  constructor(public value: string) {}

  equals(other: ValueObject): boolean {
    if (other instanceof Nickname) {
      return this.value === other.value;
    }
    return false;
  }
}

export class User {
  constructor(
    public id: ObjectID,
    public username: Username,
    public hostname: string | null,
    public password: string | null,
    public nickname: Nickname,
    public bio: string,
    public url: string | null,
    public privateKey: string | null,
    public publicKey: string | null,
    public createdAt: Clock,
    public deletedAt: Clock | null
  ) {}
}
