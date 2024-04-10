import { useCallback, useEffect, useState } from "react";
import { useRequestContext } from "../requester";
import CreatePost from "../components/CreatePost";

export function Timeline() {
  const req = useRequestContext();

  const [posts, setPosts] = useState();
  const reloadPosts = useCallback(() => {
    (async () => {
      const res = await req.get("/timeline");
      console.log(res);
    })();
  }, [req]);

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
    </div>
  );
}
