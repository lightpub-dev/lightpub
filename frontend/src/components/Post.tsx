import { Col, Row } from "antd";
import { List } from "immutable";
import React, { useMemo } from "react";
import * as styles from "./Post.module.css";
import { Link } from "react-router-dom";

export interface PostEntry {
  id: string;
  author: {
    id: string;
    username: string;
    nickname: string;
    host?: string;
  };
  content: string;
  createdAt: Date;
}

export function formatUsernameAndHost(username: string, host?: string) {
  if (host) {
    return `@${username}@${host}`;
  }
  return `@${username}`;
}

export function Post({ post }: { post: PostEntry }) {
  const formattedUsername = useMemo(() => {
    return formatUsernameAndHost(post.author.username, post.author.host);
  }, [post.author.username, post.author.host]);

  return (
    <div className={styles.post}>
      <Row>
        <Col>
          <Link
            className={styles["post-author"]}
            to={`/user/${post.author.id}`}
          >
            {post.author.nickname} ({formattedUsername})
          </Link>
        </Col>
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
