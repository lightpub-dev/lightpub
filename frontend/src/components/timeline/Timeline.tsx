import { useSelector } from "react-redux";
import { selectAuthorization } from "../../stores/authSlice";
import useSWR from "swr";
import { authedFetcher } from "../../hooks";
import PostView from "../post/PostView";
import axios from "axios";

type PostResponse = {
  id: string;
  uri: string;
  author: {
    id: string;
    uri: string;
    username: string;
    host: string | null;
    nickname: string;
  };
  content: string | null;
  privacy: "public" | "unlisted" | "follower" | "private";
  repost_of_id: string | null;
  repost_of_uri: string | null;
  reply_to_id: string | null;
  reply_to_uri: string | null;
  created_at: string;
  mentioned_users: any[];
  counts: {
    reactions: Record<string, number>;
    replies: number;
    reposts: number;
    quotes: number;
  };
  reposted_by_you: boolean | null;
  favorited_by_you: boolean | null;
  bookmarked_by_you: boolean | null;
};

type TimelineResponse = {
  next: string | null;
  result: PostResponse[];
};

export default function Timeline() {
  const authorization = useSelector(selectAuthorization);

  const { data, error, isLoading } = useSWR(
    [authorization, "/timeline"],
    authedFetcher<TimelineResponse>,
    {
      refreshInterval: 5000,
    }
  );

  if (error) {
    return <div>failed to load</div>;
  }

  if (isLoading) {
    return <div>loading...</div>;
  }

  return (
    <div>
      {data?.result.map((p) => {
        return (
          <NetworkPostView
            key={p.id}
            id={p.id}
            repost_of_id={p.repost_of_id ?? undefined}
            nickname={p.author.nickname}
            username={p.author.username}
            hostname={p.author.host}
            content={p.content}
            timestamp={new Date(p.created_at)}
          />
        );
      })}
    </div>
  );
}

function NetworkPostView({
  id,
  repost_of_id,
  nickname,
  username,
  hostname,
  content,
  timestamp,
}: {
  id: string;
  repost_of_id?: string;
  nickname: string;
  username: string;
  hostname: string | null;
  content: string | null;
  timestamp: Date;
}) {
  const authorization = useSelector(selectAuthorization);
  const { data, error, isLoading } = useSWR(
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

  if (repost_of_id === undefined) {
    return (
      <PostView
        id={id}
        nickname={nickname}
        username={username}
        hostname={hostname}
        content={content!}
        timestamp={timestamp}
      />
    );
  }

  if (isLoading) {
    return <div>Loading...</div>;
  }
  if (error) {
    return <div>error</div>;
  }

  if (data) {
    return (
      <PostView
        id={id}
        reposter={{
          nickname: nickname,
          username: username,
          hostname: hostname,
        }}
        nickname={data.author.nickname}
        username={data.author.username}
        hostname={data.author.host}
        content={data.content!}
        timestamp={new Date(data.created_at)}
      />
    );
  }

  return <div>data not found</div>;
}
