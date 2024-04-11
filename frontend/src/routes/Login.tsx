import { Button, Col, Input, Row } from "antd";
import { useRequestContext } from "../requester";
import { useCallback, useState } from "react";
import { useNavigate } from "react-router-dom";

export function LoginView() {
  const req = useRequestContext();
  const navigate = useNavigate();

  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [isLoggingIn, setIsLoggingIn] = useState(false);

  const onLogin = useCallback(() => {
    setIsLoggingIn(true);
    req
      .post("/login", {
        username,
        password,
      })
      .then((res) => {
        setIsLoggingIn(false);
        req.setBearerToken(res.data.token);
        console.log(res);
        navigate("/timeline");
      })
      .catch((err) => {
        setIsLoggingIn(false);
        req.setBearerToken(null);
        console.error(err);
        alert("Login failed");
      });
  }, [req, username, password]);

  return (
    <div>
      <Row justify="center">
        <Col span={18}>
          <Input
            placeholder="Username"
            value={username}
            onChange={(ev) => setUsername(ev.target.value)}
          />
        </Col>
      </Row>
      <Row justify="center">
        <Col span={18}>
          <Input
            placeholder="Password"
            type="password"
            value={password}
            onChange={(ev) => setPassword(ev.target.value)}
          />
        </Col>
      </Row>
      <Row justify="center">
        <Col span={18}>
          <Button type="primary" onClick={onLogin} disabled={isLoggingIn}>
            Login
          </Button>
        </Col>
      </Row>
    </div>
  );
}
