import { ObjectID } from "../domain/model/object_id";
import { Post } from "../domain/model/post";

export interface PostRepository {
  findById(id: ObjectID): Promise<Post | null>;
  save(post: Post): Promise<void>;
  update(post: Post): Promise<void>;
}
