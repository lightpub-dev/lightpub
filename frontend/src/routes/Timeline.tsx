import { useCallback, useEffect, useMemo, useState } from "react";
import { useRequestContext } from "../requester";
import CreatePost from "../components/CreatePost";
import { PostEntry, PostList } from "../components/Post";
import { List } from "immutable";

export function Timeline() {
  const req = useRequestContext();

  const [rawPosts, setRawPosts] = useState();
  const reloadPosts = useCallback(() => {
    (async () => {
      const res = await req.get("/timeline");
      setRawPosts(res.data.result);
    })();
  }, [req]);

  const posts: List<PostEntry> = useMemo(() => {
    if (!rawPosts) {
      return List();
    }

    return List(rawPosts).map((post: any) => {
      return {
        id: post.id,
        author: {
          id: post.author.id,
          username: post.author.username,
          nickname: post.author.nickname,
        },
        content: post.content,
        createdAt: new Date(post.created_at),
      };
    });
  }, [rawPosts]);

  // initialize post list
  useEffect(() => {
    reloadPosts();
  }, [reloadPosts]);

  // reload timeline when new post is created
  const onPostCreate = useCallback(() => {
    reloadPosts();
  }, [reloadPosts]);

  return (
    <div>
      <CreatePost onPostCreate={onPostCreate} />
      <PostList posts={posts} />
    </div>
  );
}
