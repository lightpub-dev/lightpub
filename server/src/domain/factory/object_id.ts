import { injectable } from "tsyringe";
import { ObjectID } from "../model/object_id";
import { v7 as uuidv7 } from "uuid";

export interface IIDGenerator {
  generate(): ObjectID;
}

@injectable()
export class DefaultIDGenerator implements IIDGenerator {
  generate(): ObjectID {
    const str = uuidv7();
    return new ObjectID(str.replaceAll("-", ""));
  }
}
