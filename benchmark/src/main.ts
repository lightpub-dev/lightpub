import { faker } from "@faker-js/faker";
import { followUser, loginUser, post, registerUser } from "./api";
import { RegisterRequest } from "./models";

const UserCount = 100;
const FollowCount = 50;
const PostsPerUser = 100;

function fakeUser(): RegisterRequest {
  return {
    username: faker.internet.userName(),
    nickname: faker.person.firstName(),
    password: faker.internet.password(),
  };
}

function fakeUsers(): RegisterRequest[] {
  const users: RegisterRequest[] = [];
  for (let i = 0; i < UserCount; i++) {
    users.push(fakeUser());
  }
  return users;
}

function chooseRandom<T>(arr: T[]): T {
  return arr[Math.floor(Math.random() * arr.length)];
}

function chooseAtRate<T>(arr: T[], rate: number): T[] {
  const chosen: T[] = [];
  for (const item of arr) {
    if (Math.random() < rate) {
      chosen.push(item);
    }
  }
  return chosen;
}

async function main() {
  const registers = fakeUsers();
  const loginUsers: {
    username: string;
    password: string;
    token: string;
  }[] = [];
  for (const reg of registers) {
    await registerUser(reg);
    const token = await loginUser(reg);
    loginUsers.push({
      username: reg.username,
      password: reg.password,
      token: token.token,
    });
  }

  for (let i = 0; i < loginUsers.length; i++) {
    const user = loginUsers[i];
    const followees = chooseAtRate(loginUsers, FollowCount / loginUsers.length);
    for (const followee of followees) {
      if (followee.username === user.username) {
        continue;
      }
      await followUser({
        target: followee.username,
        token: user.token,
      });
    }
  }

  const postCount = loginUsers.length * PostsPerUser;
  const posts: { poster: string; content: string; privacy: string }[] = [];
  for (let i = 0; i < postCount; i++) {
    const user = chooseRandom(loginUsers);
    const privacy = chooseRandom(["public", "unlisted"] as const);
    const content = faker.lorem.sentence();
    const postReq = {
      token: user.token,
      content,
      privacy,
    };
    await post(postReq);
    posts.push({
      poster: user.username,
      content,
      privacy,
    });
  }
}

main();
