import { Button, Col, Input, Row } from "antd";
import { useRequestContext } from "../requester";
import { useCallback, useState } from "react";
import { useNavigate } from "react-router-dom";

export function RegisterView() {
  const req = useRequestContext();
  const navigate = useNavigate();

  const [username, setUsername] = useState("");
  const [nickname, setNickname] = useState("");
  const [password, setPassword] = useState("");
  const [isRegistering, setIsRegistering] = useState(false);

  const onRegister = useCallback(() => {
    setIsRegistering(true);
    req
      .post("/register", {
        username,
        nickname,
        password,
      })
      .then((res) => {
        setIsRegistering(false);
        console.log(res);
        navigate("/login");
      })
      .catch((err) => {
        setIsRegistering(false);
        console.error(err);
        alert("Failed to register");
      });
  }, [req, username, nickname, password]);

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
            placeholder="Nickname"
            value={nickname}
            onChange={(ev) => setNickname(ev.target.value)}
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
          <Button type="primary" onClick={onRegister} disabled={isRegistering}>
            Register
          </Button>
        </Col>
      </Row>
    </div>
  );
}
