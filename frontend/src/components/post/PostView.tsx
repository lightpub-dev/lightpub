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
import { useCallback, useContext, useMemo, useRef } from "react";
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
import { Link, useNavigate } from "react-router-dom";
import { CreatePostContext } from "../../contexts/CreatePostContext";

export default function PostView({
  id,
  reposter,
  replyTo,
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
  replyTo?: {
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

  const repliedAuthorAtHostname = useMemo(() => {
    if (replyTo?.hostname == null) return "";
    return `@${replyTo.hostname}`;
  }, [replyTo]);

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
  const repliedProfilePage = useMemo(() => {
    if (!replyTo) return null;
    let url = `/user/@${replyTo.username}`;
    if (replyTo.hostname) {
      url += `@${replyTo.hostname}`;
    }
    return url;
  }, [replyTo]);
  const jumpToReposter = useCallback(() => {
    if (reposterProfilePage) navigate(reposterProfilePage);
  }, [navigate, reposterProfilePage]);
  const jumpToAuthor = useCallback(() => {
    navigate(authorProfilePage);
  }, [navigate, authorProfilePage]);
  const jumpToRepliedAuthor = useCallback(() => {
    if (repliedProfilePage) navigate(repliedProfilePage);
  }, [navigate, repliedProfilePage]);

  // reply button
  const createPostContext = useContext(CreatePostContext);
  const onReplyClick = useCallback(() => {
    if (!createPostContext) return;
    createPostContext.showCreatePost({
      reply_to_id: id,
    });
  }, [createPostContext, id]);

  // detail view
  const detailViewUrl = useMemo(() => {
    return `post/${id}`;
  }, [id]);
  const jumpToDetailView = useCallback(() => {
    navigate(detailViewUrl);
  }, [detailViewUrl]);

  return (
    <Box p="6" boxShadow="md" borderRadius="md" borderWidth="1px">
      <Stack spacing={3}>
        {reposter && (
          <Flex alignItems="center" fontSize="small">
            <Text marginRight="1em">リポスター: </Text>
            <Text
              fontWeight="bold"
              mr="2"
              cursor="pointer"
              onClick={jumpToReposter}
            >
              {reposter.nickname}
            </Text>
            <Text color="gray.500" cursor="pointer" onClick={jumpToReposter}>
              (@{reposter.username}
              {reposterAtHostname})
            </Text>
          </Flex>
        )}
        {replyTo && (
          <Flex alignItems="center" fontSize="small">
            <Text marginRight="1em">返信者: </Text>
            <Text
              fontWeight="bold"
              mr="2"
              cursor="pointer"
              onClick={jumpToRepliedAuthor}
            >
              {replyTo.nickname}
            </Text>
            <Text
              color="gray.500"
              cursor="pointer"
              onClick={jumpToRepliedAuthor}
            >
              (@{replyTo.username}
              {repliedAuthorAtHostname})
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
              <MenuItem onClick={jumpToDetailView}>
                <Text>詳細</Text>
              </MenuItem>

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
          <IconButton
            aria-label="Reply"
            icon={<FaReply />}
            variant="ghost"
            onClick={() => {
              onReplyClick();
            }}
          />
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
            icon={<FaHeart color={isFavoritedByMe ? "red" : "black"} />}
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
