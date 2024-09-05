import { ObjectID } from "../domain/model/object_id";
import { User } from "../domain/model/user";

export interface IUserRepository {
  save(user: User): Promise<void>;
  findById(id: ObjectID): Promise<User | null>;
  findByUsernameAndHostname(
    username: string,
    hostname: string | null
  ): Promise<User | null>;
}
