export type RegisterRequest = {
  username: string;
  nickname: string;
  password: string;
};

export type LoginRequest = {
  username: string;
  password: string;
};

export type AuthHeader = {
  token: string;
};

export type PostRequest = {
  content: string;
  privacy: 0 | 1 | 2 | 3;
} & AuthHeader;

export type ReplyRequest = PostRequest & {
  replyTo: string;
};

export type RepostRequest = Omit<PostRequest, "content"> & {
  repostOf: string;
};

export type QuoteRequest = PostRequest & {
  repostOf: string;
};

export type FollowRequest = {
  target: string;
} & AuthHeader;
