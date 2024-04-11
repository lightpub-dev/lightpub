import React, { useCallback, useEffect, useState } from "react";
import { useRequestContext } from "../requester";
import { useLoaderData } from "react-router-dom";
import { Button, Col, Row } from "antd";

interface UserProfileModel {
  id: string;
  // username: string;
}

export async function loader({ params }: { params: { userId: string } }) {
  return { userId: params.userId };
}

export function UserProfile() {
  const { userId } = useLoaderData() as { userId: string };
  const req = useRequestContext();

  const [user, setUser] = useState<UserProfileModel | null>(null);
  useEffect(() => {
    req
      .get(`/user/${userId}`)
      .then((res) => setUser(res.data))
      .catch((err) => console.error(err));
  }, [userId, req]);

  const followUser = useCallback(() => {
    req.put(`/user/${userId}/follow`, {}).catch((err) => console.error(err));
  }, [req, userId]);
  const unfollowUser = useCallback(() => {
    req.delete(`/user/${userId}/follow`, {}).catch((err) => console.error(err));
  }, [req, userId]);

  return (
    <div>
      <Row>
        <Col span={24}>{user?.id ?? "loading..."}</Col>
      </Row>
      {/* <Row>
        <Col span={12}>{user?.</Col>
      </Row> */}
      <Row>
        <Col span={8}>
          <Button type="primary" onClick={followUser}>
            Follow
          </Button>
        </Col>
        <Col span={8}>
          <Button type="primary" danger onClick={unfollowUser}>
            Unfollow
          </Button>
        </Col>
      </Row>
    </div>
  );
}
