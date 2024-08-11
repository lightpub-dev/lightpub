import { useNavigate } from "react-router-dom";
import NewPostDialog from "../../components/post/NewPostDialog";
import Sidebar from "../../components/sidebar/Sidebar";
import {
  Modal,
  ModalBody,
  ModalCloseButton,
  ModalContent,
  ModalHeader,
  ModalOverlay,
  useDisclosure,
} from "@chakra-ui/react";
import { ReactNode, useState } from "react";
import {
  CreatePostContext,
  CreatePostContextType,
} from "../../contexts/CreatePostContext";

function MainPage({ children }: { children: ReactNode }) {
  const { isOpen, onOpen, onClose } = useDisclosure();

  const navigate = useNavigate();

  const [replyToId, setReplyToId] = useState<undefined | string>(undefined);
  const createPostContext: CreatePostContextType = {
    showCreatePost: (options) => {
      setReplyToId(options.reply_to_id);
      onOpen();
    },
  };

  return (
    <>
      <CreatePostContext.Provider value={createPostContext}>
        <div className={"flex"}>
          <Sidebar
            children={<div className={"flex flex-col"}>{children}</div>}
            onItemClick={(id) => {
              switch (id) {
                case "new-post":
                  setReplyToId(undefined);
                  onOpen();
                  break;
                case "home":
                  navigate("/");
                  break;
              }
            }}
          />
        </div>
        <Modal isOpen={isOpen} onClose={onClose}>
          <ModalOverlay />
          <ModalContent>
            <ModalHeader>新規ポスト作成</ModalHeader>
            <ModalCloseButton />
            <ModalBody>
              <NewPostDialog
                onPostFinished={() => {
                  onClose();
                }}
                replyToId={replyToId}
              />
            </ModalBody>
          </ModalContent>
        </Modal>
      </CreatePostContext.Provider>
    </>
  );
}

export default MainPage;
