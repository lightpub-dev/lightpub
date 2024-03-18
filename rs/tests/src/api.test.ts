import axios from "axios";
import expect from "expect.js";
import { exec } from "child_process";
import mocha from "mocha";

const BASE_URL = "https://lightpub.tinax.local";
axios.defaults.baseURL = BASE_URL;

const truncateDB = () => {
    // execute truncate_db.sh
    return new Promise((resolve, reject) => {
        const proc = exec("bash ./truncate_db.sh", (error, stdout, stderr) => {
            console.log("stdout:", stdout);
            console.log("stderr:", stderr);
            if (error) {
                console.error(`exec error: ${error}`);
                reject(error);
                return;
            }
        });
        proc.on("exit", (code) => {
            if (code === 0) {
                console.log("Truncate DB success");
                resolve(null);
            } else {
                console.error("Truncate DB failed");
                reject();
            }
        });
    });
};

const createAndLoginUser = async (
    username: string,
    password: string
): Promise<string> => {
    const res = await axios.post(BASE_URL + "/register", {
        username,
        nickname: username,
        password,
    });
    expect(res.status).equal(200);

    const token = await axios.post(BASE_URL + "/login", {
        username,
        password,
    });
    expect(token.status).equal(200);
    expect(token.data).to.have.property("token");
    return token.data.token;
};

function authHeader(token: string) {
    return {
        headers: { Authorization: "Bearer " + token },
    };
}

describe("/register", function () {
    beforeEach(async function () {
        await truncateDB();
    });
    it("registering a new user", async function () {
        this.timeout(30000);
        const response = await axios.post(
            BASE_URL + "/register",
            {
                username: "initialuser",
                nickname: "initialnick",
                password: "password",
            },
            {
                timeout: 30000,
            }
        );
        expect(response.status).equal(200);
        expect(response.data).have.property("user_id");
    });
    it("registering duplicate users", async function () {
        this.timeout(60000);
        {
            const response = await axios.post(
                BASE_URL + "/register",
                {
                    username: "duplicateduser",
                    nickname: "duplicatednick",
                    password: "password",
                },
                {
                    timeout: 30000,
                }
            );
            expect(response.status).equal(200);
            expect(response.data).have.property("user_id");
        }
        {
            const response = await axios.post(
                BASE_URL + "/register",
                {
                    username: "duplicateduser",
                    nickname: "duplicatednick",
                    password: "password",
                },
                {
                    timeout: 30000,
                    validateStatus: () => true,
                }
            );
            expect(response.status).equal(400);
        }
    });
});

describe("/login", function () {
    before(async function () {
        this.timeout(30000);
        try {
            await truncateDB();
            const response = await axios.post(
                BASE_URL + "/register",
                {
                    username: "initialuser",
                    nickname: "initialnick",
                    password: "password",
                },
                {
                    timeout: 30000,
                }
            );
            expect(response.status).equal(200);
            expect(response.data).have.property("user_id");
            console.log("registered initial user successfully.");
        } catch (e) {
            console.error(e);
        }
    });
    it("can login with correct credentials", async function () {
        const response = await axios.post(BASE_URL + "/login", {
            username: "initialuser",
            password: "password",
        });
        expect(response.status).equal(200);
        expect(response.data).have.property("token");
    });
    it("reject login with wrong credentials", async function () {
        const response = await axios.post(
            BASE_URL + "/login",
            {
                username: "initialuser",
                password: "wrongpassword",
            },
            {
                validateStatus(status) {
                    return true;
                },
            }
        );
        expect(response.status).equal(401);
    });
    it("reject login with non-existing user", async function () {
        const response = await axios.post(
            BASE_URL + "/login",
            {
                username: "nonexistinguser",
                password: "password",
            },
            {
                validateStatus: () => true,
            }
        );
        expect(response.status).equal(401);
    });
});

describe("/post", function () {
    let token: string, token2: string;
    before(async function () {
        this.timeout(60000);
        await truncateDB();
        token = await createAndLoginUser("testuser", "password");
        token2 = await createAndLoginUser("testuser2", "password");
    });
    describe("normal post", function () {
        it("can create a public post", async function () {
            const res = await axios.post(
                BASE_URL + "/post",
                {
                    content: "public content",
                    privacy: "public",
                },
                {
                    ...authHeader(token),
                }
            );
            expect(res.status).equal(200);
            expect(res.data).have.property("post_id");
        });
        it("can create an unlisted post", async function () {
            const res = await axios.post(
                BASE_URL + "/post",
                {
                    content: "unlisted content",
                    privacy: "unlisted",
                },
                {
                    ...authHeader(token),
                }
            );
            expect(res.status).equal(200);
            expect(res.data).have.property("post_id");
        });
        it("can create a follower-only post", async function () {
            const res = await axios.post(
                BASE_URL + "/post",
                {
                    content: "follower content",
                    privacy: "follower",
                },
                {
                    ...authHeader(token),
                }
            );
            expect(res.status).equal(200);
            expect(res.data).have.property("post_id");
        });
        it("can create a private post", async function () {
            const res = await axios.post(
                BASE_URL + "/post",
                {
                    content: "private content",
                    privacy: "private",
                },
                {
                    ...authHeader(token),
                }
            );
            expect(res.status).equal(200);
            expect(res.data).have.property("post_id");
        });
    });
    describe("reply", function () {
        let publicParentId: string,
            followerParentId: string,
            privateParentId: string;
        this.beforeAll(async function () {
            this.timeout(30000);

            const res = await axios.post(
                BASE_URL + "/post",
                {
                    content: "parent post",
                    privacy: "public",
                },
                {
                    ...authHeader(token2),
                }
            );
            expect(res.status).equal(200);
            publicParentId = res.data.post_id;
            const res2 = await axios.post(
                BASE_URL + "/post",
                {
                    content: "parent post",
                    privacy: "follower",
                },
                {
                    ...authHeader(token2),
                }
            );
            expect(res2.status).equal(200);
            followerParentId = res2.data.post_id;
            const res3 = await axios.post(
                BASE_URL + "/post",
                {
                    content: "parent post",
                    privacy: "private",
                },
                {
                    ...authHeader(token2),
                }
            );
            expect(res3.status).equal(200);
            privateParentId = res3.data.post_id;
        });
        it("can make a public reply to a public post", async function () {
            const res = await axios.post(
                BASE_URL + "/post",
                {
                    content: "reply to public",
                    privacy: "public",
                    reply_to_id: publicParentId,
                },
                authHeader(token)
            );
            expect(res.status).equal(200);
            expect(res.data).have.property("post_id");
        });
        it("cannot make a public reply to a follower-only post (by non-follower)", async function () {
            const res = await axios.post(
                BASE_URL + "/post",
                {
                    content: "reply to follower",
                    privacy: "public",
                    reply_to_id: followerParentId,
                },
                {
                    validateStatus(status) {
                        return true;
                    },
                    ...authHeader(token),
                }
            );
            expect(res.status).equal(404);
        });
        it("cannot make a public reply to a private post", async function () {
            const res = await axios.post(
                BASE_URL + "/post",
                {
                    content: "reply to private",
                    privacy: "public",
                    reply_to_id: privateParentId,
                },
                {
                    validateStatus(status) {
                        return true;
                    },
                    ...authHeader(token),
                }
            );
            expect(res.status).equal(404);
        });
        it("can make a private reply to a public post", async function () {
            const res = await axios.post(
                BASE_URL + "/post",
                {
                    content: "reply to public",
                    privacy: "private",
                    reply_to_id: publicParentId,
                },
                authHeader(token)
            );
            expect(res.status).equal(200);
            expect(res.data).have.property("post_id");
        });
    });
    describe("repost", function () {
        let otherPublicParentId: string,
            otherFollowerParentId: string,
            otherPrivateParentId: string,
            myPublicParentId: string,
            myFollowerParentId: string,
            myPrivateParentId: string;
        this.beforeAll(async function () {
            this.timeout(30000);

            const res = await axios.post(
                BASE_URL + "/post",
                {
                    content: "parent post",
                    privacy: "public",
                },
                {
                    ...authHeader(token2),
                }
            );
            expect(res.status).equal(200);
            otherPublicParentId = res.data.post_id;
            const res2 = await axios.post(
                BASE_URL + "/post",
                {
                    content: "parent post",
                    privacy: "follower",
                },
                {
                    ...authHeader(token2),
                }
            );
            expect(res2.status).equal(200);
            otherFollowerParentId = res2.data.post_id;
            const res3 = await axios.post(
                BASE_URL + "/post",
                {
                    content: "parent post",
                    privacy: "private",
                },
                {
                    ...authHeader(token2),
                }
            );
            expect(res3.status).equal(200);
            otherPrivateParentId = res3.data.post_id;

            const res4 = await axios.post(
                BASE_URL + "/post",
                {
                    content: "parent post",
                    privacy: "public",
                },
                {
                    ...authHeader(token),
                }
            );
            expect(res4.status).equal(200);
            myPublicParentId = res4.data.post_id;
            const res5 = await axios.post(
                BASE_URL + "/post",
                {
                    content: "parent post",
                    privacy: "follower",
                },
                {
                    ...authHeader(token),
                }
            );
            expect(res5.status).equal(200);
            myFollowerParentId = res5.data.post_id;
            const res6 = await axios.post(
                BASE_URL + "/post",
                {
                    content: "parent post",
                    privacy: "private",
                },
                {
                    ...authHeader(token),
                }
            );
            expect(res6.status).equal(200);
            myPrivateParentId = res6.data.post_id;
        });
        it("can make a public repost of a public post", async function () {
            const res = await axios.post(
                BASE_URL + "/post",
                {
                    privacy: "public",
                    repost_of_id: otherPublicParentId,
                },
                authHeader(token)
            );
            expect(res.status).equal(200);
            expect(res.data).have.property("post_id");
        });
        it("cannot make a public repost of a follower-only post", async function () {
            const res = await axios.post(
                BASE_URL + "/post",
                {
                    privacy: "public",
                    repost_of_id: otherFollowerParentId,
                },
                {
                    validateStatus(status) {
                        return true;
                    },
                    ...authHeader(token),
                }
            );
            expect(res.status).equal(404);
        });
        it("cannot make a public repost of a private post", async function () {
            const res = await axios.post(
                BASE_URL + "/post",
                {
                    privacy: "public",
                    repost_of_id: otherPrivateParentId,
                },
                {
                    validateStatus(status) {
                        return true;
                    },
                    ...authHeader(token),
                }
            );
            expect(res.status).equal(404);
        });
        it("can make a public repost of my public post", async function () {
            const res = await axios.post(
                BASE_URL + "/post",
                {
                    privacy: "public",
                    repost_of_id: myPublicParentId,
                },
                authHeader(token)
            );
            expect(res.status).equal(200);
            expect(res.data).have.property("post_id");
        });
        it("cannot make a public repost of my follower-only post", async function () {
            const res = await axios.post(
                BASE_URL + "/post",
                {
                    privacy: "public",
                    repost_of_id: myFollowerParentId,
                },
                {
                    validateStatus(status) {
                        return true;
                    },
                    ...authHeader(token),
                }
            );
            expect(res.status).equal(404);
        });
        it("cannot make a public repost of my private post", async function () {
            const res = await axios.post(
                BASE_URL + "/post",
                {
                    privacy: "public",
                    repost_of_id: myPrivateParentId,
                },
                {
                    validateStatus(status) {
                        return true;
                    },
                    ...authHeader(token),
                }
            );
            expect(res.status).equal(404);
        });
        it("can make an unlisted repost of my public post", async function () {
            const res = await axios.post(
                BASE_URL + "/post",
                {
                    privacy: "unlisted",
                    repost_of_id: myPublicParentId,
                },
                authHeader(token)
            );
            expect(res.status).equal(200);
            expect(res.data).have.property("post_id");
        });
        it("cannot make a follower-only repost of my public post", async function () {
            const res = await axios.post(
                BASE_URL + "/post",
                {
                    privacy: "follower",
                    repost_of_id: myPublicParentId,
                },
                {
                    validateStatus(status) {
                        return true;
                    },
                    ...authHeader(token),
                }
            );
            expect(res.status).equal(400);
        });
        it("cannot make a private repost of my public post", async function () {
            const res = await axios.post(
                BASE_URL + "/post",
                {
                    privacy: "private",
                    repost_of_id: myPublicParentId,
                },
                {
                    validateStatus(status) {
                        return true;
                    },
                    ...authHeader(token),
                }
            );
            expect(res.status).equal(400);
        });
    });
});

describe("/post/{id}", function () {
    let token: string, tokenOthers: string;
    let public_post_id: string,
        follower_post_id: string,
        private_post_id: string,
        others_follower_post_id: string,
        others_private_post_id: string;
    before(async function () {
        this.timeout(60000);
        await truncateDB();
        token = await createAndLoginUser("testuser", "password");
        tokenOthers = await createAndLoginUser("testuser2", "password");

        const public_res = await axios.post(
            BASE_URL + "/post",
            {
                privacy: "public",
                content: "public sample",
            },
            {
                ...authHeader(token),
            }
        );
        expect(public_res.status).equal(200);
        public_post_id = public_res.data.post_id;

        const follower_res = await axios.post(
            BASE_URL + "/post",
            {
                privacy: "follower",
                content: "follower sample",
            },
            authHeader(token)
        );
        expect(follower_res.status).equal(200);
        follower_post_id = follower_res.data.post_id;

        const private_res = await axios.post(
            BASE_URL + "/post",
            {
                privacy: "private",
                content: "private sample",
            },
            authHeader(token)
        );
        expect(private_res.status).equal(200);
        private_post_id = private_res.data.post_id;

        const others_follower_res = await axios.post(
            BASE_URL + "/post",
            {
                privacy: "follower",
                content: "others follower sample",
            },
            authHeader(tokenOthers)
        );
        expect(others_follower_res.status).equal(200);
        others_follower_post_id = others_follower_res.data.post_id;

        const others_private_res = await axios.post(
            BASE_URL + "/post",
            {
                privacy: "private",
                content: "others private sample",
            },
            authHeader(tokenOthers)
        );
        expect(others_private_res.status).equal(200);
        others_private_post_id = others_private_res.data.post_id;
    });
    it("can get a public post", async function () {
        const res = await axios.get(BASE_URL + "/post/" + public_post_id, {
            ...authHeader(token),
        });
        expect(res.status).equal(200);
        expect(res.data).have.property("id");
        expect(res.data.id).equal(public_post_id);
        expect(res.data.content).equal("public sample");
    });
    it("can get my follower post", async function () {
        const res = await axios.get(BASE_URL + "/post/" + follower_post_id, {
            ...authHeader(token),
        });
        expect(res.status).equal(200);
        expect(res.data).have.property("id");
        expect(res.data.id).equal(follower_post_id);
        expect(res.data.content).equal("follower sample");
    });
    it("can get my private post", async function () {
        const res = await axios.get(BASE_URL + "/post/" + private_post_id, {
            ...authHeader(token),
        });
        expect(res.status).equal(200);
        expect(res.data).have.property("id");
        expect(res.data.id).equal(private_post_id);
        expect(res.data.content).equal("private sample");
    });
    it("cannot get others follower post (by non-follower)", async function () {
        const res = await axios.get(
            BASE_URL + "/post/" + others_follower_post_id,
            {
                validateStatus(status) {
                    return true;
                },
                ...authHeader(token),
            }
        );
        expect(res.status).equal(404);
    });
    it("cannot get others private post", async function () {
        const res = await axios.get(
            BASE_URL + "/post/" + others_private_post_id,
            {
                validateStatus(status) {
                    return true;
                },
                ...authHeader(token),
            }
        );
        expect(res.status).equal(404);
    });
});

describe("/follow", function () {
    let userToken1: string, userToken2: string;
    before(async function () {
        this.timeout(60000);
        await truncateDB();
        userToken1 = await createAndLoginUser("user1", "password");
        userToken2 = await createAndLoginUser("user2", "password");
    });
    describe("follow and unfollow", function () {
        it("can follow a user", async function () {
            const res = await axios.put(
                "/user/@user2/follow",
                {},
                authHeader(userToken1)
            );
            expect(res.status).equal(200);
        });
        it("can unfollow a user", async function () {
            const res = await axios.delete(
                "/user/@user2/follow",
                authHeader(userToken1)
            );
            expect(res.status).equal(200);
        });
    });
    describe("follow list", function () {
        before(async function () {
            // user1 follows user2
            // but user2 does not follow user1
            const res = await axios.put(
                "/user/@user2/follow",
                {},
                authHeader(userToken1)
            );
            expect(res.status).equal(200);

            // user2 unfollows user1
            const res2 = await axios.delete(
                "/user/@user1/follow",
                authHeader(userToken2)
            );
            expect(res2.status).equal(200);
        });
        it("can get followers", async function () {
            const res = await axios.get(
                "/user/@user2/followers",
                authHeader(userToken2)
            );
            expect(res.status).equal(200);
            expect(res.data).to.have.property("result");
            expect(res.data.result).to.have.length(1);
            expect(res.data.result[0]).to.have.property("username", "user1");

            const res2 = await axios.get(
                "/user/@user1/followers",
                authHeader(userToken1)
            );
            expect(res2.status).equal(200);
            expect(res2.data).to.have.property("result");
            expect(res2.data.result).to.have.length(0);
        });
        it("can get following", async function () {
            const res = await axios.get(
                "/user/@user1/following",
                authHeader(userToken1)
            );
            expect(res.status).equal(200);
            expect(res.data).to.have.property("result");
            expect(res.data.result).to.have.length(1);
            expect(res.data.result[0]).to.have.property("username", "user2");

            // Testing for user2's followings
            const res2 = await axios.get(
                "/user/@user2/following",
                authHeader(userToken2)
            );
            expect(res2.status).equal(200);
            expect(res2.data).to.have.property("result");
            expect(res2.data.result).to.have.length(0);
        });
    });
});

describe.only("user posts", function () {
    let userToken1: string,
        userToken2: string,
        userToken3: string,
        userToken4: string;
    before(async function () {
        this.timeout(60000);
        await truncateDB();
        [userToken1, userToken2, userToken3, userToken4] = await Promise.all([
            createAndLoginUser("user1", "password"),
            createAndLoginUser("user2", "password"),
            createAndLoginUser("user3", "password"),
            createAndLoginUser("user4", "password"),
        ]);

        // user3 follows user1
        const res = await axios.put(
            "/user/@user1/follow",
            {},
            authHeader(userToken3)
        );
        expect(res.status).equal(200);

        const publicPost = await axios.post(
            "/post",
            {
                content: "public content",
                privacy: "public",
            },
            authHeader(userToken1)
        );
        expect(publicPost.status).equal(200);

        const unlistedPost = await axios.post(
            "/post",
            {
                content: "unlisted content",
                privacy: "unlisted",
            },
            authHeader(userToken1)
        );
        expect(unlistedPost.status).equal(200);

        const followerOnlyPost = await axios.post(
            "/post",
            {
                content: "follower content",
                privacy: "follower",
            },
            authHeader(userToken1)
        );
        expect(followerOnlyPost.status).equal(200);

        const privatePost = await axios.post(
            "/post",
            {
                content: "private content @user4",
                privacy: "private",
            },
            authHeader(userToken1)
        );
        expect(privatePost.status).equal(200);
    });
    it("poster can see their own posts", async function () {
        const res = await axios.get(
            "/user/@user1/posts",
            authHeader(userToken1)
        );
        expect(res.status).equal(200);
        expect(res.data).to.have.property("result");
        expect(res.data.result).to.have.length(4);
        expect(res.data.result[3]).have.property("content", "public content");
        expect(res.data.result[2]).have.property("content", "unlisted content");
        expect(res.data.result[1]).have.property("content", "follower content");
        expect(res.data.result[0]).have.property(
            "content",
            "private content @user4"
        );
    });
    it("non-poster can see public and unlisted posts", async function () {
        const res = await axios.get(
            "/user/@user1/posts",
            authHeader(userToken2)
        );
        expect(res.status).equal(200);
        expect(res.data).to.have.property("result");
        expect(res.data.result).to.have.length(2);
        expect(res.data.result[1]).have.property("content", "public content");
        expect(res.data.result[0]).have.property("content", "unlisted content");
    });
    it("follower can see follower post", async function () {
        const res = await axios.get(
            "/user/@user1/posts",
            authHeader(userToken3)
        );
        expect(res.status).equal(200);
        expect(res.data).to.have.property("result");
        expect(res.data.result).to.have.length(3);
        expect(res.data.result[2]).have.property("content", "public content");
        expect(res.data.result[1]).have.property("content", "unlisted content");
        expect(res.data.result[0]).have.property("content", "follower content");
    });
    it("mentioned user can see private post", async function () {
        const res = await axios.get(
            "/user/@user1/posts",
            authHeader(userToken4)
        );
        expect(res.status).equal(200);
        expect(res.data).to.have.property("result");
        expect(res.data.result).to.have.length(3);
        expect(res.data.result[2]).have.property("content", "public content");
        expect(res.data.result[1]).have.property("content", "unlisted content");
        expect(res.data.result[0]).have.property(
            "content",
            "private content @user4"
        );
    });
});
