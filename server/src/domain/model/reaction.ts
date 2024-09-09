import { ObjectID } from "./object_id";

export class PostFavorite {
  constructor(
    public postId: ObjectID,
    public userId: ObjectID,
    public favoritedAt: Date,
    public id?: number
  ) {}
}

export class PostBookmark {
  constructor(
    public postId: ObjectID,
    public userId: ObjectID,
    public bookmarkedAt: Date,
    public id?: number
  ) {}
}

export class PostReaction {
  constructor(
    public postId: ObjectID,
    public userId: ObjectID,
    public reaction: string,
    public reactedAt: Date,
    public id?: number
  ) {}
}
