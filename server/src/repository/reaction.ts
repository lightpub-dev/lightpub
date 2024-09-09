import { ObjectID } from "../domain/model/object_id";
import {
  PostBookmark,
  PostFavorite,
  PostReaction,
} from "../domain/model/reaction";

export interface ReactionRepository {
  save(reaction: PostFavorite | PostBookmark | PostReaction): Promise<void>;
  delete(reaction: PostFavorite | PostBookmark | PostReaction): Promise<void>;
  find(
    reactionType: "favorite",
    userId: ObjectID,
    postId: ObjectID
  ): Promise<PostFavorite | null>;
  find(
    reactionType: "bookmark",
    userId: ObjectID,
    postId: ObjectID
  ): Promise<PostBookmark | null>;
  find(
    reactionType: "reaction",
    userId: ObjectID,
    postId: ObjectID
  ): Promise<PostReaction[]>;
  find(
    reactionType: "reaction",
    userId: ObjectID,
    postId: ObjectID,
    emoji: string
  ): Promise<PostReaction | null>;
}
