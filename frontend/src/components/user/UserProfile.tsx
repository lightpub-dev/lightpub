import {
  Box,
  Avatar,
  Heading,
  Text,
  Button,
  VStack,
  HStack,
} from "@chakra-ui/react";
import PostView from "../post/PostView";
import { useMemo } from "react";
import { useAppSelector } from "../../hooks";
import { selectUsername } from "../../stores/authSlice";

interface UserProfileProps {
  username: string;
  hostname?: string;
  nickname: string;
  bio?: string;
  posts: Array<any>; // Assuming PostView component accepts an array of post objects
}

function UserProfile({
  username,
  hostname,
  nickname,
  bio,
  posts,
}: UserProfileProps) {
  const atHostname = useMemo(() => {
    if (hostname) return `@${hostname}`;
    return "";
  }, [hostname]);

  const bioText = useMemo(() => {
    if (bio) return bio;
    return "このユーザは自己紹介を登録していません。";
  }, [bio]);

  const currentUsername = useAppSelector(selectUsername);
  const showFollowButton = useMemo(() => {
    return (
      hostname !== undefined || // remote user
      currentUsername !== username // different user
    );
  }, [username, hostname]);

  return (
    <Box p={5} shadow="md" borderWidth="1px" borderRadius="lg">
      <VStack spacing={4} align="start">
        <HStack spacing={4}>
          <Avatar
            size="xl"
            name={username}
            src="https://via.placeholder.com/150"
          />
          <Box>
            <Heading as="h2" size="lg">
              {nickname}
            </Heading>
            <Text fontSize="sm" color="gray.500">
              {username}
              {atHostname}
            </Text>
          </Box>
        </HStack>
        <HStack spacing={4}>
          <Button
            colorScheme="blue"
            display={showFollowButton ? "inherit" : "none"}
          >
            Follow
          </Button>
          <Box>
            <Text>{bioText}</Text>
          </Box>
        </HStack>
      </VStack>
      <VStack spacing={4} mt={5} align="start">
        {posts.map((post, index) => (
          <PostView key={index} {...post} />
        ))}
      </VStack>
    </Box>
  );
}

export default UserProfile;
