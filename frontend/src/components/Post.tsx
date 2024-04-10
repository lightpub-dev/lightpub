import { Col, Row } from "antd";
import { List } from "immutable";
import React from "react";

export interface PostEntry {
  id: string;
  author: {
    id: string;
    username: string;
    nickname: string;
  };
  content: string;
  createdAt: Date;
}

export function Post({ post }: { post: PostEntry }) {
  return (
    <div>
      <Row>
        <Col>{post.author.nickname}</Col>
      </Row>
      <Row>
        <Col>{post.content}</Col>
      </Row>
      <Row>
        <Col>{post.createdAt.toLocaleString()}</Col>
      </Row>
    </div>
  );
}

export function PostList({ posts }: { posts: List<PostEntry> }) {
  return (
    <div>
      {posts.map((post) => (
        <Post key={post.id} post={post} />
      ))}
    </div>
  );
}
