import {
  Box,
  Flex,
  IconButton,
  Text,
  Stack,
  Menu,
  MenuItem,
  MenuList,
  MenuButton,
  AlertDialog,
  AlertDialogBody,
  AlertDialogContent,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogOverlay,
  Button,
  useDisclosure,
} from "@chakra-ui/react";
import { useCallback, useMemo, useRef } from "react";
import {
  FaReply,
  FaRetweet,
  FaHeart,
  FaSmile,
  FaEllipsisV,
} from "react-icons/fa";
import { useSelector } from "react-redux";
import { selectAuthorization } from "../../stores/authSlice";
import axios from "axios";
import { useNavigate } from "react-router-dom";

export default function PostView({
  id,
  reposter,
  nickname,
  username,
  hostname,
  content,
  timestamp: timestampObj,
  isFavoritedByMe,
  isBookmarkedByMe,
}: {
  id: string;
  reposter?: {
    nickname: string;
    username: string;
    hostname: string | null;
  };
  nickname: string;
  username: string;
  hostname: string | null;
  content: string;
  timestamp: Date;
  isFavoritedByMe?: boolean;
  isBookmarkedByMe?: boolean;
}) {
  const timestamp = timestampObj.toLocaleString();

  const atHostname = useMemo(() => {
    if (hostname === null) {
      return "";
    }
    return `@${hostname}`;
  }, [hostname]);

  const reposterAtHostname = useMemo(() => {
    if (reposter?.hostname == null) return "";
    return `@${reposter.hostname}`;
  }, [reposter]);

  const authorization = useSelector(selectAuthorization);

  // delete
  const {
    isOpen: isDeleteOpen,
    onOpen: onDeleteOpen,
    onClose: onDeleteClose,
  } = useDisclosure();
  const deleteCancelRef = useRef<HTMLButtonElement>(null);
  const deletePost = useCallback(async () => {
    await axios.delete("/post/" + id, {
      headers: {
        authorization,
      },
    });
  }, [authorization, id]);

  // repost
  const repostPost = useCallback(async () => {
    try {
      await axios.post(
        "/post",
        {
          privacy: "public",
          repost_of_id: id,
        },
        {
          headers: {
            authorization,
          },
        }
      );
    } catch (ex: any) {
      console.warn(ex.response);
      alert("リポスト失敗");
    }
  }, [authorization, id]);

  // favorite
  const favoritePost = useCallback(async () => {
    if (isFavoritedByMe === undefined) {
      return;
    }
    try {
      if (!isFavoritedByMe) {
        await axios.put(`/post/${id}/favorite`, null, {
          headers: {
            authorization,
          },
        });
      } else {
        await axios.delete(`/post/${id}/favorite`, {
          headers: {
            authorization,
          },
        });
      }
    } catch (ex: any) {
      console.warn(ex.response);
      alert("お気に入り失敗");
    }
  }, [authorization, id, isFavoritedByMe]);

  // bookmark
  const bookmarkPost = useCallback(async () => {
    if (isBookmarkedByMe === undefined) {
      return;
    }
    try {
      if (!isBookmarkedByMe) {
        await axios.put(`/post/${id}/bookmark`, null, {
          headers: {
            authorization,
          },
        });
      } else {
        await axios.delete(`/post/${id}/bookmark`, {
          headers: {
            authorization,
          },
        });
      }
    } catch (ex: any) {
      console.warn(ex.response);
      alert("ブックマーク失敗");
    }
  }, [authorization, id, isBookmarkedByMe]);
  const bookmarkToggleText = useMemo(() => {
    if (!isBookmarkedByMe) {
      return "ブックマークに追加";
    } else {
      return "ブックマークから削除";
    }
  }, [isBookmarkedByMe]);

  // profile page jump
  const navigate = useNavigate();
  const reposterProfilePage = useMemo(() => {
    if (!reposter) return null;
    let url = `/user/@${reposter.username}`;
    if (reposter.hostname) {
      url += `@${reposter.hostname}`;
    }
    return url;
  }, [reposter]);
  const authorProfilePage = useMemo(() => {
    let url = `/user/@${username}`;
    if (hostname) {
      url += `@${hostname}`;
    }
    return url;
  }, [username, hostname]);
  const jumpToReposter = useCallback(() => {
    if (reposterProfilePage) navigate(reposterProfilePage);
  }, [navigate, reposterProfilePage]);
  const jumpToAuthor = useCallback(() => {
    navigate(authorProfilePage);
  }, [navigate, authorProfilePage]);

  return (
    <Box p="6" boxShadow="md" borderRadius="md" borderWidth="1px">
      <Stack spacing={3}>
        {reposter && (
          <Flex alignItems="center">
            <Text>
              <pre>Reposted by </pre>
            </Text>
            <Text
              fontWeight="bold"
              mr="2"
              cursor="pointer"
              onClick={jumpToReposter}
            >
              {reposter.nickname}
            </Text>
            <Text color="gray.500" cursor="pointer" onClick={jumpToReposter}>
              (@{username}
              {reposterAtHostname})
            </Text>
          </Flex>
        )}
        <Flex alignItems="center" justify="space-between">
          <Flex alignItems="center">
            <Text
              fontWeight="bold"
              mr="2"
              cursor="pointer"
              onClick={jumpToAuthor}
            >
              {nickname}
            </Text>
            <Text color="gray.500" cursor="pointer" onClick={jumpToAuthor}>
              (@{username}
              {atHostname})
            </Text>
          </Flex>
          <Menu>
            <MenuButton
              as={IconButton}
              aria-label="Options"
              icon={<FaEllipsisV />}
              variant="ghost"
            />
            <MenuList>
              <MenuItem
                onClick={() => {
                  bookmarkPost();
                }}
              >
                <Text>{bookmarkToggleText}</Text>
              </MenuItem>
              <MenuItem
                onClick={() => {
                  onDeleteOpen();
                }}
              >
                <Text color="red">削除</Text>
              </MenuItem>
            </MenuList>
          </Menu>
        </Flex>
        <Text>{content}</Text>
        <Text color="gray.500" fontSize="sm">
          {timestamp}
        </Text>
        <Flex justify="space-between" mt="4">
          <IconButton aria-label="Reply" icon={<FaReply />} variant="ghost" />
          <IconButton
            aria-label="Repost"
            icon={<FaRetweet />}
            variant="ghost"
            onClick={() => {
              repostPost();
            }}
          />
          <IconButton
            aria-label="Favorite"
            icon={<FaHeart />}
            variant="ghost"
            onClick={() => {
              favoritePost();
            }}
          />
          <IconButton
            aria-label="Emoji Reaction"
            icon={<FaSmile />}
            variant="ghost"
          />
        </Flex>
      </Stack>
      <AlertDialog
        isOpen={isDeleteOpen}
        leastDestructiveRef={deleteCancelRef}
        onClose={onDeleteClose}
      >
        <AlertDialogOverlay>
          <AlertDialogContent>
            <AlertDialogHeader fontSize="lg" fontWeight="bold">
              ポストを削除する
            </AlertDialogHeader>

            <AlertDialogBody>本当にポストを削除しますか?</AlertDialogBody>

            <AlertDialogFooter>
              <Button ref={deleteCancelRef} onClick={onDeleteClose}>
                キャンセル
              </Button>
              <Button
                colorScheme="red"
                onClick={() => {
                  deletePost();
                  onDeleteClose();
                }}
                ml={3}
              >
                削除する
              </Button>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialogOverlay>
      </AlertDialog>
    </Box>
  );
}
