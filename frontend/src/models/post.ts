export type PostResponse = {
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

export function convertReactions(
  reactions: Record<string, number>
): Array<{ emoji: string; count: number }> {
  const list = [];
  for (let key in reactions) {
    list.push({
      emoji: key,
      count: reactions[key],
    });
  }
  return list;
}
