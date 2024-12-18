import { useSelector } from "react-redux";
import { selectAuthorization } from "../../stores/authSlice";
import useSWR from "swr";
import { authedFetcher, cookieFetcher } from "../../hooks";
import { convertReactions, PostResponse } from "../../models/post";
import { NetworkPostView } from "../post/NetworkPostView";

type TimelineResponse = {
  next: string | null;
  result: PostResponse[];
};

export default function Timeline() {
  const authorization = useSelector(selectAuthorization);

  const { data, error, isLoading } = useSWR(
    "/timeline",
    cookieFetcher<TimelineResponse>,
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
            reply_to_id={p.reply_to_id ?? undefined}
            nickname={p.author.nickname}
            username={p.author.username}
            hostname={p.author.host}
            content={p.content}
            timestamp={new Date(p.created_at)}
            reactions={convertReactions(p.counts.reactions)}
            isFavoritedByMe={p.favorited_by_you ?? undefined}
            isBookmarkedByMe={p.bookmarked_by_you ?? undefined}
          />
        );
      })}
    </div>
  );
}
