import { useSelector } from "react-redux";
import useSWR from "swr";
import { selectAuthorization } from "../../stores/authSlice";
import { convertReactions, PostResponse } from "../../models/post";
import axios from "axios";
import PostView from "./PostView";

export function NetworkPostView({
  id,
  repost_of_id,
  reply_to_id,
  nickname,
  username,
  hostname,
  content,
  timestamp,
  reactions,
  isFavoritedByMe,
  isBookmarkedByMe,
}: {
  id: string;
  repost_of_id?: string;
  reply_to_id?: string;
  nickname: string;
  username: string;
  hostname: string | null;
  content: string | null;
  timestamp: Date;
  reactions: { emoji: string; count: number }[];
  isFavoritedByMe?: boolean;
  isBookmarkedByMe?: boolean;
}) {
  const authorization = useSelector(selectAuthorization);
  const {
    data: repostData,
    error: repostError,
    isLoading: repostIsLoading,
  } = useSWR(
    () => (repost_of_id === undefined ? null : [authorization, repost_of_id]),
    ([authorization, repost_of_id]: [
      string,
      string,
    ]): Promise<PostResponse> => {
      return axios
        .get(`/post/${repost_of_id}`, {
          headers: {
            authorization,
          },
        })
        .then((res) => res.data as PostResponse);
    },
    {
      revalidateOnFocus: false,
    }
  );
  const {
    data: replyData,
    error: replyError,
    isLoading: replyIsLoading,
  } = useSWR(
    () => (reply_to_id === undefined ? null : [authorization, reply_to_id]),
    ([authorization, reply_to_id]: [string, string]): Promise<PostResponse> => {
      return axios
        .get(`/post/${reply_to_id}`, {
          headers: {
            authorization,
          },
        })
        .then((res) => res.data as PostResponse);
    },
    {
      revalidateOnFocus: false,
    }
  );

  if (repost_of_id === undefined && reply_to_id === undefined) {
    return (
      <PostView
        id={id}
        displayedPostId={id}
        nickname={nickname}
        username={username}
        hostname={hostname}
        content={content!}
        timestamp={timestamp}
        isFavoritedByMe={isFavoritedByMe}
        isBookmarkedByMe={isBookmarkedByMe}
        reactions={reactions}
      />
    );
  }

  if (repostIsLoading || replyIsLoading) {
    return <div>Loading...</div>;
  }
  if (repostError || replyError) {
    return <div>error</div>;
  }

  if (repostData) {
    return (
      <PostView
        id={id}
        displayedPostId={repostData.id} // リポストされた投稿のID
        reposter={{
          nickname: nickname,
          username: username,
          hostname: hostname,
        }}
        nickname={repostData.author.nickname}
        username={repostData.author.username}
        hostname={repostData.author.host}
        content={repostData.content!}
        timestamp={new Date(repostData.created_at)}
        isFavoritedByMe={repostData.favorited_by_you ?? undefined}
        isBookmarkedByMe={repostData.bookmarked_by_you ?? undefined}
        reactions={convertReactions(repostData.counts.reactions)}
      />
    );
  }

  if (replyData) {
    return (
      <PostView
        id={id}
        displayedPostId={id}
        replyTo={{
          nickname: replyData.author.nickname,
          username: replyData.author.username,
          hostname: replyData.author.host,
        }}
        nickname={nickname}
        username={username}
        hostname={hostname}
        content={content!}
        timestamp={timestamp}
        reactions={reactions}
        isFavoritedByMe={isFavoritedByMe}
        isBookmarkedByMe={isBookmarkedByMe}
      />
    );
  }

  return <div>data not found</div>;
}
