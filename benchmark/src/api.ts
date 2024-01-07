import axios from "axios";
import {
  AuthHeader,
  FollowRequest,
  LoginRequest,
  PostRequest,
  QuoteRequest,
  RegisterRequest,
  ReplyRequest,
  RepostRequest,
} from "./models";

axios.defaults.baseURL = "http://localhost:1323";

export async function registerUser(reg: RegisterRequest) {
  await axios.post("/register", {
    username: reg.username,
    nickname: reg.nickname,
    password: reg.password,
  });
}

export async function loginUser(login: LoginRequest): Promise<AuthHeader> {
  const res = await axios.post("/login", {
    username: login.username,
    password: login.password,
  });
  return {
    token: res.data.token,
  };
}

export async function post(
  post: PostRequest | QuoteRequest | ReplyRequest | RepostRequest
) {
  const token = post.token;

  const body = {
    content: "content" in post ? post.content : undefined,
    privacy: post.privacy,
  };

  let url = "/post";
  if ("replyTo" in post) {
    url = `/post/${post.replyTo}/reply`;
  } else if ("repostOf" in post) {
    url = `/post/${post.repostOf}/repost`;
  } else if ("quoteOf" in post) {
    url = `/post/${post.quoteOf}/quote`;
  }

  await axios.post(url, body, {
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });
}

export async function followUser(info: FollowRequest) {
  await axios.put(`/user/@${info.target}/follow`, undefined, {
    headers: {
      Authorization: `Bearer ${info.token}`,
    },
  });
}
