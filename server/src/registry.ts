import { container } from "tsyringe";

import { DefaultUserFactory } from "./domain/factory/user";
import { DefaultIDGenerator } from "./domain/factory/object_id";
import { UserSqliteRepository } from "./repository/sqlite/user";
import {
  ID_GENERATOR,
  SECRET_REPOSITORY,
  USER_FACTORY,
  USER_REPOSITORY,
} from "./registry_key";
import { SecretSqliteRepository } from "./repository/sqlite/secret";

export function registerSqliteServices() {
  container.register(USER_FACTORY, {
    useClass: DefaultUserFactory,
  });
  container.register(ID_GENERATOR, {
    useClass: DefaultIDGenerator,
  });

  container.register(USER_REPOSITORY, {
    useClass: UserSqliteRepository,
  });
  container.register(SECRET_REPOSITORY, {
    useClass: SecretSqliteRepository,
  });
}
