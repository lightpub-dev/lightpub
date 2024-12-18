import { ReactNode, useCallback, useMemo } from "react";
import {
  IconButton,
  Avatar,
  Box,
  CloseButton,
  Flex,
  HStack,
  VStack,
  Icon,
  useColorModeValue,
  Link,
  Drawer,
  DrawerContent,
  Text,
  useDisclosure,
  BoxProps,
  FlexProps,
  Menu,
  MenuButton,
  MenuDivider,
  MenuItem,
  MenuList,
  Spacer,
  Image,
} from "@chakra-ui/react";
import {
  FiHome,
  FiTrendingUp,
  FiSettings,
  FiMenu,
  FiBell,
  FiChevronDown,
} from "react-icons/fi";
import { IoAddOutline } from "react-icons/io5";
import { IconType } from "react-icons";
import { ReactText } from "react";
import { RiTimeLine } from "react-icons/ri";

import logo from "../../assets/lightpub.svg";
import { authedFetcher, useAppSelector } from "../../hooks";
import { selectAuthorization, selectUsername } from "../../stores/authSlice";
import useSWR from "swr";
import { UserResponse } from "../../models/user";
import { useNavigate } from "react-router-dom";
import axios from "axios";

type LinkItemId = "new-post" | "home" | "trending" | "settings";

interface LinkItemProps {
  id: LinkItemId;
  name: string;
  icon: IconType;
}

const LinkItems: Array<LinkItemProps> = [
  { id: "new-post", name: "New Post", icon: IoAddOutline },
  { id: "home", name: "Home", icon: FiHome },
  { id: "trending", name: "Trending", icon: FiTrendingUp },
  { id: "settings", name: "Settings", icon: FiSettings },
];

export default function Sidebar({
  children,
  onItemClick,
}: {
  children: ReactNode;
  onItemClick?: (id: LinkItemId) => void;
}) {
  const { isOpen, onOpen, onClose } = useDisclosure();

  const onClick = useCallback(
    (id: LinkItemId) => {
      if (onItemClick) onItemClick(id);
    },
    [onItemClick]
  );

  return (
    <Box minH="100vh" bg={useColorModeValue("gray.100", "gray.900")}>
      <SidebarContent
        onClose={() => onClose}
        display={{ base: "none", md: "block" }}
        onItemClick={onClick}
      />
      <Drawer
        autoFocus={false}
        isOpen={isOpen}
        placement="left"
        onClose={onClose}
        returnFocusOnClose={false}
        onOverlayClick={onClose}
        size="full"
      >
        <DrawerContent>
          <SidebarContent onClose={onClose} onItemClick={onClick} />
        </DrawerContent>
      </Drawer>
      {/* mobilenav */}
      <MobileNav onOpen={onOpen} />
      <Box ml={{ base: 0, md: 60 }} p="4">
        {children}
      </Box>
    </Box>
  );
}

interface SidebarProps extends BoxProps {
  onClose: () => void;
  onItemClick: (id: LinkItemId) => void;
}

const SidebarContent = ({ onClose, onItemClick, ...rest }: SidebarProps) => {
  return (
    <Box
      transition="3s ease"
      bg={useColorModeValue("white", "gray.900")}
      borderRight="1px"
      borderRightColor={useColorModeValue("gray.200", "gray.700")}
      w={{ base: "full", md: 60 }}
      pos="fixed"
      h="full"
      {...rest}
    >
      <Flex h="20" alignItems="center" mx="8" justifyContent="center">
        <Image src={logo} alt="logo" maxHeight={"100%"} />
        <CloseButton display={{ base: "flex", md: "none" }} onClick={onClose} />
      </Flex>
      {LinkItems.map((link) => (
        <NavItem
          key={link.name}
          icon={link.icon}
          onClick={() => {
            // console.log("clicked: " + link.id);
            onItemClick(link.id);
          }}
        >
          {link.name}
        </NavItem>
      ))}
    </Box>
  );
};

interface NavItemProps extends FlexProps {
  icon: IconType;
  children: ReactText;
}

const NavItem = ({ icon, children, ...rest }: NavItemProps) => {
  return (
    <Link
      href="#"
      style={{ textDecoration: "none" }}
      _focus={{ boxShadow: "none" }}
    >
      <Flex
        align="center"
        p="4"
        mx="4"
        borderRadius="lg"
        role="group"
        cursor="pointer"
        _hover={{
          bg: "cyan.400",
          color: "white",
        }}
        {...rest}
      >
        {icon && (
          <Icon
            mr="4"
            fontSize="16"
            _groupHover={{
              color: "white",
            }}
            as={icon}
          />
        )}
        {children}
      </Flex>
    </Link>
  );
};

interface MobileProps extends FlexProps {
  onOpen: () => void;
}

const MobileNav = ({ onOpen, ...rest }: MobileProps) => {
  const authorization = useAppSelector(selectAuthorization);
  const username = useAppSelector(selectUsername);
  const { data, error, isLoading } = useSWR(
    username === null || authorization === null
      ? null
      : [authorization, `/user/@${username}`],
    authedFetcher<UserResponse>,
    {
      refreshInterval: 30000,
    }
  );

  const userProfile = useMemo(() => {
    if (error) {
      console.error("my profile fetch error");
      console.error(error);
      return null;
    }
    if (isLoading || !data) {
      return null;
    }

    return {
      username: data.username,
      nickname: data.nickname,
    };
  }, [data, error, isLoading]);

  const navigate = useNavigate();

  const onLoginOrLogout = useCallback(() => {
    if (authorization === null) {
      navigate("/login");
    } else {
      axios
        .post("/logout")
        .then(() => {
          console.log("logout success");
          navigate("/login");
        })
        .catch((e) => {
          console.error("logout failed");
          console.error(e);
        });
    }
  }, [navigate]);

  return (
    <Flex
      ml={{ base: 0, md: 60 }}
      px={{ base: 4, md: 4 }}
      height="20"
      alignItems="center"
      bg={useColorModeValue("white", "gray.900")}
      borderBottomWidth="1px"
      borderBottomColor={useColorModeValue("gray.200", "gray.700")}
      align="center"
      {...rest}
    >
      <IconButton
        display={{ base: "flex", md: "none" }}
        onClick={onOpen}
        variant="outline"
        aria-label="open menu"
        icon={<FiMenu />}
      />

      <Spacer />
      <HStack spacing={{ base: "0", md: "6" }}>
        <IconButton
          size="lg"
          variant="ghost"
          aria-label="open menu"
          icon={<RiTimeLine />}
        />
        <IconButton
          size="lg"
          variant="ghost"
          aria-label="open menu"
          icon={<FiBell />}
        />
      </HStack>
      <Spacer />
      <Flex alignItems={"center"}>
        <Menu>
          <MenuButton
            py={2}
            transition="all 0.3s"
            _focus={{ boxShadow: "none" }}
          >
            <HStack>
              <Avatar
                size={"sm"}
                src={"https://avatars.githubusercontent.com/u/41512077"}
              />
              <VStack
                display={{ base: "none", md: "flex" }}
                alignItems="flex-start"
                spacing="1px"
                ml="2"
              >
                {userProfile ? (
                  <>
                    <Text fontSize="sm">{userProfile.nickname}</Text>
                    <Text fontSize="xs" color="gray.600">
                      @{userProfile.username}
                    </Text>
                  </>
                ) : (
                  <Text fontSize="xs" color="gray.600">
                    未ログイン
                  </Text>
                )}
              </VStack>
              <Box display={{ base: "none", md: "flex" }}>
                <FiChevronDown />
              </Box>
            </HStack>
          </MenuButton>
          <MenuList
            bg={useColorModeValue("white", "gray.900")}
            borderColor={useColorModeValue("gray.200", "gray.700")}
          >
            <MenuItem>Profile</MenuItem>
            <MenuItem>Settings</MenuItem>
            <MenuItem>Billing</MenuItem>
            <MenuDivider />
            <MenuItem onClick={onLoginOrLogout}>
              {authorization === null ? "ログイン" : "ログアウト"}
            </MenuItem>
          </MenuList>
        </Menu>
      </Flex>
    </Flex>
  );
};
