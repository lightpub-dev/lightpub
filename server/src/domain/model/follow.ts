import { Clock } from "../../utils/clock";
import { ObjectID } from "./object_id";

export class UserFollow {
  constructor(
    public id: bigint | null,
    public readonly followerId: ObjectID,
    public readonly followeeId: ObjectID,
    public readonly createdAt: Clock
  ) {}

  setID(id: number | bigint) {
    this.id = BigInt(id);
  }
}
