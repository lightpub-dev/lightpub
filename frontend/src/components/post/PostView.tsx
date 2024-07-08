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

export default function PostView({
  id,
  nickname,
  username,
  hostname,
  content,
  timestamp: timestampObj,
}: {
  id: string;
  nickname: string;
  username: string;
  hostname: string | null;
  content: string;
  timestamp: Date;
}) {
  const timestamp = timestampObj.toLocaleString();

  const atHostname = useMemo(() => {
    if (hostname === null) {
      return "";
    }
    return `@${hostname}`;
  }, [hostname]);

  const authorization = useSelector(selectAuthorization);

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

  return (
    <Box p="6" boxShadow="md" borderRadius="md" borderWidth="1px">
      <Stack spacing={3}>
        <Flex alignItems="center" justify="space-between">
          <Flex alignItems="center">
            <Text fontWeight="bold" mr="2">
              {nickname}
            </Text>
            <Text color="gray.500">
              @{username}
              {atHostname}
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
              <MenuItem>
                <Text>ブックマークに追加</Text>
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
          />
          <IconButton
            aria-label="Favorite"
            icon={<FaHeart />}
            variant="ghost"
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
