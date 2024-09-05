import { ValueObject } from "../../utils/eq";

export class ObjectID implements ValueObject {
  public readonly id: string;

  constructor(id: string) {
    if (id.includes("-")) {
      throw new Error("Invalid ObjectID");
    }
    this.id = id;
  }

  equals(other: ValueObject): boolean {
    if (other instanceof ObjectID) {
      return this.id === other.id;
    }
    return false;
  }
}
