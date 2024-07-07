import React, { useState } from "react";
import {
  Box,
  Button,
  FormControl,
  FormLabel,
  Input,
  Stack,
  Text,
} from "@chakra-ui/react";
import axios from "axios";
import { useNavigate } from "react-router-dom";

const RegistrationPage: React.FC = () => {
  const [username, setUsername] = useState("");
  const [nickname, setNickname] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");

  const navigate = useNavigate();

  const handleRegister = async () => {
    // console.log("Register button pressed", { username, nickname, password });
    try {
      await axios.post("/register", {
        username: username,
        nickname: nickname,
        password: password,
      });
      setError("");
      // move to login page
      navigate("/login");
    } catch (ex: any) {
      console.warn(ex);
      setError(ex.response.data);
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
      <Box width="300px" p="8" boxShadow="lg" borderRadius="md">
        <form>
          <Stack spacing={4}>
            <FormControl id="username">
              <FormLabel>Username</FormLabel>
              <Input
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
              />
            </FormControl>
            <FormControl id="nickname">
              <FormLabel>Nickname</FormLabel>
              <Input
                type="text"
                value={nickname}
                onChange={(e) => setNickname(e.target.value)}
              />
            </FormControl>
            <FormControl id="password">
              <FormLabel>Password</FormLabel>
              <Input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
              />
            </FormControl>
            {error && <Text color="red.500">{error}</Text>}
            <Button colorScheme="blue" onClick={handleRegister}>
              Register
            </Button>
          </Stack>
        </form>
      </Box>
    </Box>
  );
};

export default RegistrationPage;
