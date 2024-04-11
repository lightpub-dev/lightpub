import React, { useCallback, useState } from "react";
import { Modal, Form, Input, Select, Button } from "antd";
import { useRequestContext } from "../requester";

const { Option } = Select;

export function CreatePost({
  onPostCreate,
}: {
  onPostCreate?: (newPostId: string) => void;
}) {
  const [visible, setVisible] = useState(false);

  const handleOpenModal = () => {
    setVisible(true);
  };

  const handleCloseModal = () => {
    setVisible(false);
  };

  const req = useRequestContext();

  const handleCreatePost = useCallback(
    (values: {
      content: string;
      privacy: "public" | "unlisted" | "follower" | "private";
    }) => {
      console.log("Creating post:", values);
      req
        .post("/post", values)
        .then((res) => {
          setVisible(false);
          if (onPostCreate) {
            onPostCreate(res.data.post_id);
          }
        })
        .catch((err) => {
          console.error(err);
          alert("Failed to create post");
        });
    },
    [req]
  );

  return (
    <>
      <Button type="primary" onClick={handleOpenModal}>
        Create Post
      </Button>
      <Modal
        title="Create Post"
        open={visible}
        onCancel={handleCloseModal}
        footer={null}
      >
        <Form onFinish={handleCreatePost}>
          <Form.Item
            name="content"
            label="Post Content"
            rules={[{ required: true, message: "Please enter post content" }]}
          >
            <Input.TextArea rows={4} />
          </Form.Item>
          <Form.Item
            name="privacy"
            label="Post Privacy"
            rules={[{ required: true, message: "Please select post privacy" }]}
          >
            <Select>
              <Option value="public">Public</Option>
              <Option value="unlisted">Unlisted</Option>
              <Option value="follower">Follower-only</Option>
              <Option value="private">Private</Option>
            </Select>
          </Form.Item>
          <Form.Item>
            <Button type="primary" htmlType="submit">
              Send
            </Button>
          </Form.Item>
        </Form>
      </Modal>
    </>
  );
}

export default CreatePost;
