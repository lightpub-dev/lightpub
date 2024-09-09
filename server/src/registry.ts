import { container } from "tsyringe";

import { DefaultUserFactory } from "./domain/factory/user";
import { DefaultIDGenerator } from "./domain/factory/object_id";
import { UserMysqlRepository } from "./repository/mysql/user";
import {
  FOLLOW_FACTORY,
  FOLLOW_REPOSITORY,
  ID_GENERATOR,
  POST_FACTORY,
  POST_REPOSITORY,
  REACTION_REPOSITORY,
  SECRET_REPOSITORY,
  USER_FACTORY,
  USER_REPOSITORY,
} from "./registry_key";
import { SecretMysqlRepository } from "./repository/mysql/secret";
import { DefaultFollowFactory } from "./domain/factory/follow";
import { FollowMysqlRepository } from "./repository/mysql/follow";
import { DefaultPostFactory } from "./domain/factory/post";
import { PostMysqlRepository } from "./repository/mysql/post";
import { ReactionMysqlRepository } from "./repository/mysql/reaction";

export function registerMysqlServices() {
  container.register(USER_FACTORY, {
    useClass: DefaultUserFactory,
  });
  container.register(FOLLOW_FACTORY, {
    useClass: DefaultFollowFactory,
  });
  container.register(POST_FACTORY, {
    useClass: DefaultPostFactory,
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
  container.register(POST_REPOSITORY, {
    useClass: PostMysqlRepository,
  });
  container.register(REACTION_REPOSITORY, {
    useClass: ReactionMysqlRepository,
  });
}
