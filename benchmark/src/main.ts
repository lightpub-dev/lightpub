import { faker } from "@faker-js/faker";
import { followUser, loginUser, post, registerUser } from "./api";
import { RegisterRequest } from "./models";

const UserCount = 1;
const FollowCount = 0;
const PostsPerUser = 200;

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

const HashTagPoolCount = 100;
const HashTagPool: string[] = [];
for (let i = 0; i < HashTagPoolCount; i++) {
  HashTagPool.push(faker.lorem.word());
}

function fakeHashtags(): string[] {
  const count = Math.floor(Math.random() * 5);
  const hashtags: string[] = [];
  for (let i = 0; i < count; i++) {
    hashtags.push(chooseRandom(HashTagPool));
  }
  return hashtags;
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

let lastTime = Date.now();

function timeStart() {
  lastTime = Date.now();
}

function timeStopAndLog(task: string) {
  const now = Date.now();
  console.log(`${task} took ${now - lastTime}ms`);
  lastTime = now;
}

async function main() {
  const registers = fakeUsers();
  const loginUsers: {
    username: string;
    password: string;
    token: string;
  }[] = [];
  for (const reg of registers) {
    timeStart();
    await registerUser(reg);
    timeStopAndLog("register");
    timeStart();
    const token = await loginUser(reg);
    timeStopAndLog("login");
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
      timeStart();
      await followUser({
        target: followee.username,
        token: user.token,
      });
      timeStopAndLog("follow");
    }
  }

  const postCount = loginUsers.length * PostsPerUser;
  const posts: { poster: string; content: string; privacy: number }[] = [];
  for (let i = 0; i < postCount; i++) {
    const user = chooseRandom(loginUsers);
    const privacy = chooseRandom([0, 1] as const);
    let content = faker.lorem.sentence();
    const hashtags = fakeHashtags();
    for (const hashtag of hashtags) {
      content += ` #${hashtag}`;
    }
    const postReq = {
      token: user.token,
      content,
      privacy,
    };
    await post(postReq);
    timeStart();
    posts.push({
      poster: user.username,
      content,
      privacy,
    });
    timeStopAndLog("post");
  }
}

main();
