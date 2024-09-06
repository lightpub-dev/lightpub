import { container } from "tsyringe";

import { DefaultUserFactory } from "./domain/factory/user";
import { DefaultIDGenerator } from "./domain/factory/object_id";
import { UserMysqlRepository } from "./repository/mysql/user";
import {
  FOLLOW_FACTORY,
  FOLLOW_REPOSITORY,
  ID_GENERATOR,
  SECRET_REPOSITORY,
  USER_FACTORY,
  USER_REPOSITORY,
} from "./registry_key";
import { SecretMysqlRepository } from "./repository/mysql/secret";
import { DefaultFollowFactory } from "./domain/factory/follow";
import { FollowMysqlRepository } from "./repository/mysql/follow";

export function registerMysqlServices() {
  container.register(USER_FACTORY, {
    useClass: DefaultUserFactory,
  });
  container.register(FOLLOW_FACTORY, {
    useClass: DefaultFollowFactory,
  });

  container.register(ID_GENERATOR, {
    useClass: DefaultIDGenerator,
  });

  container.register(USER_REPOSITORY, {
    useClass: UserMysqlRepository,
  });
  container.register(SECRET_REPOSITORY, {
    useClass: SecretMysqlRepository,
  });
  container.register(FOLLOW_REPOSITORY, {
    useClass: FollowMysqlRepository,
  });
}
