import React, { useState } from "react";
import {
  Box,
  Button,
  FormControl,
  FormLabel,
  Select,
  Stack,
  Textarea,
} from "@chakra-ui/react";
import axios from "axios";
import { useAppSelector } from "../../hooks";
import { selectAuthorization } from "../../stores/authSlice";

const NewPostDialog: React.FC = () => {
  const [content, setContent] = useState("");
  const [visibility, setVisibility] = useState("public");
  const [postable, setPostable] = useState(true);

  const authorization = useAppSelector(selectAuthorization);

  const handlePost = async () => {
    // console.log("Post button pressed", { content, visibility });
    setPostable(false);
    try {
      await axios.post(
        "/post",
        {
          content: content,
          privacy: visibility,
        },
        {
          headers: {
            authorization,
          },
        }
      );
      setContent("");
    } catch (ex) {
      console.warn(ex);
    } finally {
      setPostable(true);
    }
  };

  return (
    <Box
      width="100vw"
      height="100vh"
      display="flex"
      alignItems="center"
      justifyContent="center"
    >
      <Box width="400px" p="8" boxShadow="lg" borderRadius="md">
        <form>
          <Stack spacing={4}>
            <FormControl id="content">
              <FormLabel>Content</FormLabel>
              <Textarea
                value={content}
                onChange={(e) => setContent(e.target.value)}
              />
            </FormControl>
            <FormControl id="visibility">
              <FormLabel>Post Visibility</FormLabel>
              <Select
                value={visibility}
                onChange={(e) => setVisibility(e.target.value)}
              >
                <option value="public">Public</option>
                <option value="unlisted">Unlisted</option>
                <option value="follower">Follower-only</option>
                <option value="private">Private</option>
              </Select>
            </FormControl>
            <Button
              disabled={!postable}
              colorScheme="blue"
              onClick={handlePost}
            >
              Post
            </Button>
          </Stack>
        </form>
      </Box>
    </Box>
  );
};

export default NewPostDialog;