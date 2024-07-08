import { useSelector } from "react-redux";
import { selectAuthorization } from "../../stores/authSlice";
import useSWR from "swr";
import { authedFetcher } from "../../hooks";
import PostView from "../post/PostView";

type TimelineResponse = {
  next: string | null;
  result: {
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
  }[];
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
          <PostView
            key={p.id}
            id={p.id}
            nickname={p.author.nickname}
            username={p.author.username}
            hostname={p.author.host}
            content={p.content!}
            timestamp={new Date(p.created_at)}
          />
        );
      })}
    </div>
  );
}
