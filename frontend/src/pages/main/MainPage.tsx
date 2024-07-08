import { useNavigate } from "react-router-dom";
import NewPostDialog from "../../components/post/NewPostDialog";
import Sidebar from "../../components/sidebar/Sidebar";
import Timeline from "../../components/timeline/Timeline";
import {
  Modal,
  ModalBody,
  ModalCloseButton,
  ModalContent,
  ModalHeader,
  ModalOverlay,
  useDisclosure,
} from "@chakra-ui/react";

function MainPage() {
  const { isOpen, onOpen, onClose } = useDisclosure();

  const navigate = useNavigate();

  return (
    <>
      <div className={"flex"}>
        <Sidebar
          children={
            <div className={"flex flex-col"}>
              <Timeline />
            </div>
          }
          onItemClick={(id) => {
            switch (id) {
              case "new-post":
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
            />
          </ModalBody>
        </ModalContent>
      </Modal>
    </>
  );
}

export default MainPage;
