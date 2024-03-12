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
import process from "process";
import http from "http";

axios.defaults.baseURL = "http://localhost:8000/api";

const httpAgent = new http.Agent({ keepAlive: true });

export async function registerUser(reg: RegisterRequest) {
  await axios.post(
    "/register",
    {
      username: reg.username,
      nickname: reg.nickname,
      password: reg.password,
    },
    {
      headers: {
        Accept: "application/json",
      },
      httpAgent: httpAgent,
    }
  );
}

export async function loginUser(login: LoginRequest): Promise<AuthHeader> {
  const res = await axios.post(
    "/login",
    {
      username: login.username,
      password: login.password,
    },
    {
      headers: {
        Accept: "application/json",
      },
      httpAgent: httpAgent,
    }
  );
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
  } as any;

  let url = "/posts";
  if ("replyTo" in post) {
    body["reply_to_id"] = post.replyTo;
  } else if ("repostOf" in post) {
    body["repost_of_id"] = post.repostOf;
  } else if ("quoteOf" in post) {
    body["repost_of_id"] = post.quoteOf;
  }

  await axios.post(url, body, {
    headers: {
      Authorization: `Bearer ${token}`,
      Accept: "application/json",
    },
    httpAgent: httpAgent,
  });
}

export async function followUser(info: FollowRequest) {
  await axios.post(
    `/follow`,
    {
      user_spec: `@${info.target}`,
    },
    {
      headers: {
        Authorization: `Bearer ${info.token}`,
        Accept: "application/json",
      },
      httpAgent: httpAgent,
    }
  );
}
