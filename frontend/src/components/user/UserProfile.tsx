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
import { useMemo, useState } from "react";
import { authedFetcher, useAppSelector } from "../../hooks";
import { selectAuthorization, selectUsername } from "../../stores/authSlice";
import axios from "axios";
import useSWR from "swr";
import { PaginatedResponse } from "../../models/pagination";
import { convertReactions, PostResponse } from "../../models/post";
import { NetworkPostView } from "../post/NetworkPostView";

interface UserProfileProps {
  username: string;
  hostname?: string;
  nickname: string;
  bio?: string;
  posts: Array<any>; // Assuming PostView component accepts an array of post objects
  is_following_you?: boolean;
  is_followed_by_you?: boolean;
}

function UserProfile({
  username,
  hostname,
  nickname,
  bio,
  posts,
  is_followed_by_you,
  is_following_you,
}: UserProfileProps) {
  // Hostname
  const atHostname = useMemo(() => {
    if (hostname) return `@${hostname}`;
    return "";
  }, [hostname]);

  // Bio
  const bioText = useMemo(() => {
    if (bio) return bio;
    return "このユーザは自己紹介を登録していません。";
  }, [bio]);

  // Current User
  const currentUsername = useAppSelector(selectUsername);
  const authorization = useAppSelector(selectAuthorization);

  // Follow Button
  const showFollowButton = useMemo(() => {
    return (
      hostname !== undefined || // remote user
      currentUsername !== username // different user
    );
  }, [username, hostname]);
  const [isFollowing, setIsFollowing] = useState(is_followed_by_you ?? false);
  const followText = isFollowing ? "Unfollow" : "Follow";
  const [isFollowProessing, setIsFollowProcessing] = useState(false);
  const onFollowClick = () => {
    if (isFollowing) {
      setIsFollowProcessing(true);
      axios
        .delete(`/user/@${username}${atHostname}/follow`, {
          headers: {
            Authorization: authorization,
          },
        })
        .then(() => {
          setIsFollowing(false);
        })
        .catch((e) => {
          console.error("unfollow error");
          console.error(e);
          alert("フォロー解除に失敗しました");
        })
        .finally(() => {
          setIsFollowProcessing(false);
        });
    } else {
      setIsFollowProcessing(true);
      axios
        .put(
          `/user/@${username}${atHostname}/follow`,
          {},
          {
            headers: {
              Authorization: authorization,
            },
          }
        )
        .then(() => {
          setIsFollowing(true);
        })
        .catch((e) => {
          console.error("follow error");
          console.error(e);
          alert("フォローに失敗しました");
        })
        .finally(() => {
          setIsFollowProcessing(false);
        });
    }
  };

  // user posts
  const userPostResult = useSWR(
    [authorization, `user/@${username}${atHostname}/posts`],
    authedFetcher<PaginatedResponse<PostResponse>>
  );
  const userPostElement = useMemo(() => {
    if (userPostResult.error) {
      console.error(userPostResult.error);
      return <p>User post fetch error</p>;
    }
    if (userPostResult.isLoading) {
      return <p>Posts loading...</p>;
    }
    if (!userPostResult.data) {
      return <p>No posts</p>;
    }

    return userPostResult.data.result.map((post) => {
      return (
        <NetworkPostView
          key={post.id}
          id={post.id}
          username={post.author.username}
          nickname={post.author.nickname}
          hostname={post.author.host}
          content={post.content}
          timestamp={new Date(post.created_at)}
          reply_to_id={post.reply_to_id ?? undefined}
          repost_of_id={post.repost_of_id ?? undefined}
          reactions={convertReactions(post.counts.reactions)}
          isBookmarkedByMe={post.bookmarked_by_you ?? undefined}
          isFavoritedByMe={post.favorited_by_you ?? undefined}
        />
      );
    });
  }, [userPostResult]);

  return (
    <>
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
              onClick={onFollowClick}
              disabled={isFollowProessing}
            >
              {followText}
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
      {userPostElement}
    </>
  );
}

export default UserProfile;
