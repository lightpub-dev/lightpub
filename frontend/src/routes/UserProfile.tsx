import React, { useEffect, useState } from "react";
import { useRequestContext } from "../requester";
import { useLoaderData } from "react-router-dom";
import { Button, Col, Row } from "antd";

interface UserProfileModel {
  id: string;
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

  return (
    <div>
      <Row>
        <Col span={24}>{user?.id ?? "loading..."}</Col>
      </Row>
      <Row>
        <Col>
          <Button type="primary">Follow</Button>
        </Col>
      </Row>
    </div>
  );
}
