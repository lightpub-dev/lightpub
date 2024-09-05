import { injectable } from "tsyringe";
import { ObjectID } from "../model/object_id";
import { v7 as uuidv7, v4 as uuidv4 } from "uuid";

export interface IIDGenerator {
  generate(): ObjectID;
  generateRandom(): ObjectID;
}

@injectable()
export class DefaultIDGenerator implements IIDGenerator {
  generate(): ObjectID {
    const str = uuidv7();
    return new ObjectID(str.replaceAll("-", ""));
  }

  generateRandom(): ObjectID {
    const str = uuidv4();
    return new ObjectID(str.replaceAll("-", ""));
  }
}
