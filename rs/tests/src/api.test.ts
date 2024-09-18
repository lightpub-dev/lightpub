import axios, { type AxiosResponse } from "axios";
import { describe, it, expect, beforeAll, beforeEach, test } from "bun:test";

const BASE_URL = "https://lightpub.tinax.local";
axios.defaults.baseURL = BASE_URL;
axios.defaults.validateStatus = () => true;

const GOOD_PASSWORD = "1234AbcD!?";

const truncateDB = async () => {
    const res = await axios.post(BASE_URL + "/debug/truncate");
    showExpect(res.status, res).toBe(200);
    console.log("truncated DB");
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
    showExpect(res.status, res).toBe(200);

    const token = await axios.post(BASE_URL + "/login", {
        username,
        password,
    });
    showExpect(token.status, res).toBe(200);
    showExpect(token.data, res).toHaveProperty("token");
    return token.data.token;
};

function authHeader(token: string) {
    return {
        headers: { Authorization: "Bearer " + token },
    };
}

function showResponse(res: any) {
    let msg = "actual response:\n";
    msg += "status: " + res.status + "\n";
    msg += JSON.stringify(res.data, null, 2);
    return msg;
}

function showExpect<T>(
    res: T | undefined | null,
    actual: AxiosResponse<any, any>
) {
    return expect(res, showResponse(actual));
}

describe("/register", () => {
    beforeEach(async () => {
        await truncateDB();
    });
    describe("success", () => {
        it(
            "registering a new user",
            async () => {
                const response = await axios.post(
                    BASE_URL + "/register",
                    {
                        username: "initialuser",
                        nickname: "initialnick",
                        password: GOOD_PASSWORD,
                    },
                    {
                        timeout: 30000,
                    }
                );
                showExpect(response.status, response).toBe(200);
                showExpect(response.data, response).toHaveProperty("user_id");
            },
            {
                timeout: 30000,
            }
        );
    });
    describe("duplicated user", () => {
        it(
            "registering duplicate users",
            async () => {
                {
                    const response = await axios.post(
                        BASE_URL + "/register",
                        {
                            username: "duplicateduser",
                            nickname: "duplicatednick",
                            password: GOOD_PASSWORD,
                        },
                        {
                            timeout: 30000,
                        }
                    );
                    showExpect(response.status, response).toBe(200);
                    showExpect(response.data, response).toHaveProperty(
                        "user_id"
                    );
                }
                {
                    const response = await axios.post(
                        BASE_URL + "/register",
                        {
                            username: "duplicateduser",
                            nickname: "duplicatednick",
                            password: GOOD_PASSWORD,
                        },
                        {
                            timeout: 30000,
                            validateStatus: () => true,
                        }
                    );
                    showExpect(response.status, response).toBe(400);
                }
            },
            {
                timeout: 60000,
            }
        );
    });
    describe("bad usernames", () => {
        async function checkFails(username: string) {
            const response = await axios.post(
                BASE_URL + "/register",
                {
                    username,
                    nickname: "nickname",
                    password: GOOD_PASSWORD,
                },
                {
                    timeout: 30000,
                    validateStatus: () => true,
                }
            );
            showExpect(response.status, response).toBe(400);
        }
        it(
            "contains kanji",
            async () => {
                await checkFails("kanji感じ");
            },
            {
                timeout: 30000,
            }
        );
        it(
            "too short",
            async () => {
                await checkFails("ab");
            },
            {
                timeout: 30000,
            }
        );
        it(
            "too long",
            async () => {
                await checkFails("123456789abcdefgh");
            },
            {
                timeout: 30000,
            }
        );
        it(
            "contains special characters",
            async () => {
                await checkFails("special!char@foobar");
            },
            {
                timeout: 30000,
            }
        );
    });
    describe("bad passwords", () => {
        async function checkFails(password: string) {
            const response = await axios.post(
                BASE_URL + "/register",
                {
                    username: "username",
                    nickname: "nickname",
                    password,
                },
                {
                    timeout: 30000,
                    validateStatus: () => true,
                }
            );
            showExpect(response.status, response).toBe(400);
        }
        it(
            "too short",
            async () => {
                await checkFails("1234Ab!");
            },
            {
                timeout: 30000,
            }
        );
        it(
            "no uppercase",
            async () => {
                await checkFails("1234abcd!?");
            },
            {
                timeout: 30000,
            }
        );
        it(
            "no lowercase",
            async () => {
                await checkFails("1234ABCD!?");
            },
            {
                timeout: 30000,
            }
        );
        it(
            "no special chars",
            async () => {
                await checkFails("1234ABCDEFgh");
            },
            {
                timeout: 30000,
            }
        );
        it(
            "too long",
            async () => {
                await checkFails(
                    "1234ABCDEojt3039a84u5v90u908!h9a8u?Fgu09ta0w85gv0a7h"
                );
            },
            {
                timeout: 30000,
            }
        );
    });
});

describe("/login", () => {
    beforeAll(async () => {
        try {
            await truncateDB();
            const response = await axios.post(
                BASE_URL + "/register",
                {
                    username: "initialuser",
                    nickname: "initialnick",
                    password: GOOD_PASSWORD,
                },
                {
                    timeout: 30000,
                }
            );
            showExpect(response.status, response).toBe(200);
            showExpect(response.data, response).toHaveProperty("user_id");
            console.log("registered initial user successfully.");
        } catch (e) {
            console.error(e);
        }
    });
    it("can login with correct credentials", async () => {
        const response = await axios.post(BASE_URL + "/login", {
            username: "initialuser",
            password: GOOD_PASSWORD,
        });
        showExpect(response.status, response).toBe(200);
        showExpect(response.data, response).toHaveProperty("token");
    });
    it("reject login with wrong credentials", async () => {
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
        showExpect(response.status, response).toBe(401);
    });
    it("reject login with non-existing user", async () => {
        const response = await axios.post(
            BASE_URL + "/login",
            {
                username: "nonexistinguser",
                password: GOOD_PASSWORD,
            },
            {
                validateStatus: () => true,
            }
        );
        showExpect(response.status, response).toBe(401);
    });
});

describe("/post", () => {
    let token: string, token2: string;
    beforeAll(async () => {
        await truncateDB();
        token = await createAndLoginUser("testuser", GOOD_PASSWORD);
        token2 = await createAndLoginUser("testuser2", GOOD_PASSWORD);
    });
    describe("normal post", () => {
        let publicPostId: string | undefined;
        it("can create a public post", async () => {
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
            showExpect(res.status, res).toBe(200);
            showExpect(res.data, res).toHaveProperty("post_id");
            publicPostId = res.data.post_id;
        });
        it("can create an unlisted post", async () => {
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
            showExpect(res.status, res).toBe(200);
            showExpect(res.data, res).toHaveProperty("post_id");
        });
        it("can create a follower-only post", async () => {
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
            showExpect(res.status, res).toBe(200);
            showExpect(res.data, res).toHaveProperty("post_id");
        });
        it("can create a private post", async () => {
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
            showExpect(res.status, res).toBe(200);
            showExpect(res.data, res).toHaveProperty("post_id");
        });
        it("can delete my post", async () => {
            if (!publicPostId) {
                expect().fail();
            }

            const res = await axios.delete(BASE_URL + "/post/" + publicPostId, {
                ...authHeader(token),
            });
            showExpect(res.status, res).toBe(200);
        });
        it("returns 404 when deleting non-existing post", async () => {
            const res = await axios.delete(
                BASE_URL + "/post/538ec871f81348c79158fecda95325e5",
                {
                    ...authHeader(token),
                    validateStatus: () => true,
                }
            );
            showExpect(res.status, res).toBe(404);
        });
    });
    describe("reply", () => {
        let publicParentId: string,
            followerParentId: string,
            privateParentId: string;
        beforeAll(async () => {
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
            showExpect(res.status, res).toBe(200);
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
            showExpect(res2.status, res).toBe(200);
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
            showExpect(res3.status, res).toBe(200);
            privateParentId = res3.data.post_id;
        });
        it("can make a public reply to a public post", async () => {
            const res = await axios.post(
                BASE_URL + "/post",
                {
                    content: "reply to public",
                    privacy: "public",
                    reply_to_id: publicParentId,
                },
                authHeader(token)
            );
            showExpect(res.status, res).toBe(200);
            showExpect(res.data, res).toHaveProperty("post_id");
        });
        it("cannot make a public reply to a follower-only post (by non-follower)", async () => {
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
            showExpect(res.status, res).toBe(404);
        });
        it("cannot make a public reply to a private post", async () => {
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
            showExpect(res.status, res).toBe(404);
        });
        it("can make a private reply to a public post", async () => {
            const res = await axios.post(
                BASE_URL + "/post",
                {
                    content: "reply to public",
                    privacy: "private",
                    reply_to_id: publicParentId,
                },
                authHeader(token)
            );
            showExpect(res.status, res).toBe(200);
            showExpect(res.data, res).toHaveProperty("post_id");
        });
    });
    describe("repost", () => {
        let otherPublicParentId: string,
            otherFollowerParentId: string,
            otherPrivateParentId: string,
            myPublicParentId: string,
            myFollowerParentId: string,
            myPrivateParentId: string;
        beforeAll(async () => {
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
            showExpect(res.status, res).toBe(200);
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
            showExpect(res2.status, res).toBe(200);
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
            showExpect(res3.status, res).toBe(200);
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
            showExpect(res4.status, res).toBe(200);
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
            showExpect(res5.status, res).toBe(200);
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
            showExpect(res6.status, res).toBe(200);
            myPrivateParentId = res6.data.post_id;
        });
        it("can make a public repost of a public post", async () => {
            const res = await axios.post(
                BASE_URL + "/post",
                {
                    privacy: "public",
                    repost_of_id: otherPublicParentId,
                },
                authHeader(token)
            );
            showExpect(res.status, res).toBe(200);
            showExpect(res.data, res).toHaveProperty("post_id");
        });
        it("cannot make a public repost of a follower-only post", async () => {
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
            showExpect(res.status, res).toBe(404);
        });
        it("cannot make a public repost of a private post", async () => {
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
            showExpect(res.status, res).toBe(404);
        });
        it("can make a public repost of my public post", async () => {
            const res = await axios.post(
                BASE_URL + "/post",
                {
                    privacy: "public",
                    repost_of_id: myPublicParentId,
                },
                authHeader(token)
            );
            showExpect(res.status, res).toBe(200);
            showExpect(res.data, res).toHaveProperty("post_id");
        });
        it("cannot make a public repost of my follower-only post", async () => {
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
            showExpect(res.status, res).toBe(404);
        });
        it("cannot make a public repost of my private post", async () => {
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
            showExpect(res.status, res).toBe(404);
        });
        it("can make an unlisted repost of my public post", async () => {
            const res = await axios.post(
                BASE_URL + "/post",
                {
                    privacy: "unlisted",
                    repost_of_id: myPublicParentId,
                },
                authHeader(token)
            );
            showExpect(res.status, res).toBe(200);
            showExpect(res.data, res).toHaveProperty("post_id");
        });
        it("cannot make a follower-only repost of my public post", async () => {
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
            showExpect(res.status, res).toBe(400);
        });
        it("cannot make a private repost of my public post", async () => {
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
            showExpect(res.status, res).toBe(400);
        });
    });
});

describe("/post/{id}", () => {
    let token: string, tokenOthers: string;
    let public_post_id: string,
        follower_post_id: string,
        private_post_id: string,
        others_follower_post_id: string,
        others_private_post_id: string;
    beforeAll(async () => {
        await truncateDB();
        token = await createAndLoginUser("testuser", GOOD_PASSWORD);
        tokenOthers = await createAndLoginUser("testuser2", GOOD_PASSWORD);

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
        showExpect(public_res.status, public_res).toBe(200);
        public_post_id = public_res.data.post_id;

        const follower_res = await axios.post(
            BASE_URL + "/post",
            {
                privacy: "follower",
                content: "follower sample",
            },
            authHeader(token)
        );
        showExpect(follower_res.status, follower_res).toBe(200);
        follower_post_id = follower_res.data.post_id;

        const private_res = await axios.post(
            BASE_URL + "/post",
            {
                privacy: "private",
                content: "private sample",
            },
            authHeader(token)
        );
        showExpect(private_res.status, private_res).toBe(200);
        private_post_id = private_res.data.post_id;

        const others_follower_res = await axios.post(
            BASE_URL + "/post",
            {
                privacy: "follower",
                content: "others follower sample",
            },
            authHeader(tokenOthers)
        );
        showExpect(others_follower_res.status, others_follower_res).toBe(200);
        others_follower_post_id = others_follower_res.data.post_id;

        const others_private_res = await axios.post(
            BASE_URL + "/post",
            {
                privacy: "private",
                content: "others private sample",
            },
            authHeader(tokenOthers)
        );
        showExpect(others_private_res.status, others_private_res).toBe(200);
        others_private_post_id = others_private_res.data.post_id;
    });
    it("can get a public post", async () => {
        const res = await axios.get(BASE_URL + "/post/" + public_post_id, {
            ...authHeader(token),
            validateStatus(status) {
                return true;
            },
        });
        showExpect(res.status, res).toBe(200);
        showExpect(res.data, res).toHaveProperty("id");
        showExpect(res.data.id, res).toBe(public_post_id);
        showExpect(res.data.content, res).toBe("public sample");
    });
    it("can get my follower post", async () => {
        const res = await axios.get(BASE_URL + "/post/" + follower_post_id, {
            ...authHeader(token),
            validateStatus(status) {
                return true;
            },
        });
        showExpect(res.status, res).toBe(200);
        showExpect(res.data, res).toHaveProperty("id");
        showExpect(res.data.id, res).toBe(follower_post_id);
        showExpect(res.data.content, res).toBe("follower sample");
    });
    it("can get my private post", async () => {
        const res = await axios.get(BASE_URL + "/post/" + private_post_id, {
            ...authHeader(token),
            validateStatus(status) {
                return true;
            },
        });
        showExpect(res.status, res).toBe(200);
        showExpect(res.data, res).toHaveProperty("id");
        showExpect(res.data.id, res).toBe(private_post_id);
        showExpect(res.data.content, res).toBe("private sample");
    });
    it("cannot get others follower post (by non-follower)", async () => {
        const res = await axios.get(
            BASE_URL + "/post/" + others_follower_post_id,
            {
                validateStatus(status) {
                    return true;
                },
                ...authHeader(token),
            }
        );
        showExpect(res.status, res).toBe(404);
    });
    it("cannot get others private post", async () => {
        const res = await axios.get(
            BASE_URL + "/post/" + others_private_post_id,
            {
                validateStatus(status) {
                    return true;
                },
                ...authHeader(token),
            }
        );
        showExpect(res.status, res).toBe(404);
    });
});

describe.only("/follow", () => {
    let userToken1: string, userToken2: string;
    beforeAll(async () => {
        await truncateDB();
        userToken1 = await createAndLoginUser("user1", GOOD_PASSWORD);
        userToken2 = await createAndLoginUser("user2", GOOD_PASSWORD);
    }, 60000);
    describe("follow and unfollow", () => {
        it("can follow a user", async () => {
            const res = await axios.put(
                "/user/@user2/follow",
                {},
                authHeader(userToken1)
            );
            showExpect(res.status, res).toBe(200);
        });
        it("can unfollow a user", async () => {
            const res = await axios.delete(
                "/user/@user2/follow",
                authHeader(userToken1)
            );
            showExpect(res.status, res).toBe(200);
        });
    });
    describe("follow list", () => {
        beforeAll(async () => {
            // user1 follows user2
            // but user2 does not follow user1
            const res = await axios.put(
                "/user/@user2/follow",
                {},
                authHeader(userToken1)
            );
            showExpect(res.status, res).toBe(200);

            // user2 unfollows user1
            const res2 = await axios.delete(
                "/user/@user1/follow",
                authHeader(userToken2)
            );
            showExpect(res2.status, res).toBe(200);
        });
        it("can get followers", async () => {
            const res = await axios.get(
                "/user/@user2/followers",
                authHeader(userToken2)
            );
            showExpect(res.status, res).toBe(200);
            showExpect(res.data, res).toHaveProperty("result");
            showExpect(res.data.result, res).toHaveLength(1);
            showExpect(res.data.result[0], res).toHaveProperty(
                "username",
                "user1"
            );

            const res2 = await axios.get(
                "/user/@user1/followers",
                authHeader(userToken1)
            );
            showExpect(res2.status, res).toBe(200);
            showExpect(res2.data, res).toHaveProperty("result");
            showExpect(res2.data.result, res).toHaveLength(0);
        });
        it("can get following", async () => {
            const res = await axios.get(
                "/user/@user1/following",
                authHeader(userToken1)
            );
            showExpect(res.status, res).toBe(200);
            showExpect(res.data, res).toHaveProperty("result");
            showExpect(res.data.result, res).toHaveLength(1);
            showExpect(res.data.result[0], res).toHaveProperty(
                "username",
                "user2"
            );

            // Testing for user2's followings
            const res2 = await axios.get(
                "/user/@user2/following",
                authHeader(userToken2)
            );
            showExpect(res2.status, res).toBe(200);
            showExpect(res2.data, res).toHaveProperty("result");
            showExpect(res2.data.result, res).toHaveLength(0);
        });
    });
});

describe("user posts", () => {
    let userToken1: string,
        userToken2: string,
        userToken3: string,
        userToken4: string;
    beforeAll(async () => {
        await truncateDB();
        [userToken1, userToken2, userToken3, userToken4] = await Promise.all([
            createAndLoginUser("user1", GOOD_PASSWORD),
            createAndLoginUser("user2", GOOD_PASSWORD),
            createAndLoginUser("user3", GOOD_PASSWORD),
            createAndLoginUser("user4", GOOD_PASSWORD),
        ]);

        // user3 follows user1
        const res = await axios.put(
            "/user/@user1/follow",
            {},
            authHeader(userToken3)
        );
        showExpect(res.status, res).toBe(200);

        const publicPost = await axios.post(
            "/post",
            {
                content: "public content",
                privacy: "public",
            },
            authHeader(userToken1)
        );
        showExpect(publicPost.status, publicPost).toBe(200);

        const unlistedPost = await axios.post(
            "/post",
            {
                content: "unlisted content",
                privacy: "unlisted",
            },
            authHeader(userToken1)
        );
        showExpect(unlistedPost.status, unlistedPost).toBe(200);

        const followerOnlyPost = await axios.post(
            "/post",
            {
                content: "follower content",
                privacy: "follower",
            },
            authHeader(userToken1)
        );
        showExpect(followerOnlyPost.status, followerOnlyPost).toBe(200);

        const privatePost = await axios.post(
            "/post",
            {
                content: "private content @user4",
                privacy: "private",
            },
            authHeader(userToken1)
        );
        showExpect(privatePost.status, privatePost).toBe(200);
    }, 60000);
    it("poster can see their own posts", async () => {
        const res = await axios.get(
            "/user/@user1/posts",
            authHeader(userToken1)
        );
        showExpect(res.status, res).toBe(200);
        showExpect(res.data, res).toHaveProperty("result");
        showExpect(res.data.result, res).toHaveLength(4);
        showExpect(res.data.result[3], res).toHaveProperty(
            "content",
            "public content"
        );
        showExpect(res.data.result[2], res).toHaveProperty(
            "content",
            "unlisted content"
        );
        showExpect(res.data.result[1], res).toHaveProperty(
            "content",
            "follower content"
        );
        showExpect(res.data.result[0], res).toHaveProperty(
            "content",
            "private content @user4"
        );
    });
    it("non-poster can see public and unlisted posts", async () => {
        const res = await axios.get(
            "/user/@user1/posts",
            authHeader(userToken2)
        );
        showExpect(res.status, res).toBe(200);
        showExpect(res.data, res).toHaveProperty("result");
        showExpect(res.data.result, res).toHaveLength(2);
        showExpect(res.data.result[1], res).toHaveProperty(
            "content",
            "public content"
        );
        showExpect(res.data.result[0], res).toHaveProperty(
            "content",
            "unlisted content"
        );
    });
    it("follower can see follower post", async () => {
        const res = await axios.get(
            "/user/@user1/posts",
            authHeader(userToken3)
        );
        showExpect(res.status, res).toBe(200);
        showExpect(res.data, res).toHaveProperty("result");
        showExpect(res.data.result, res).toHaveLength(3);
        showExpect(res.data.result[2], res).toHaveProperty(
            "content",
            "public content"
        );
        showExpect(res.data.result[1], res).toHaveProperty(
            "content",
            "unlisted content"
        );
        showExpect(res.data.result[0], res).toHaveProperty(
            "content",
            "follower content"
        );
    });
    it("mentioned user can see private post", async () => {
        const res = await axios.get(
            "/user/@user1/posts",
            authHeader(userToken4)
        );
        showExpect(res.status, res).toBe(200);
        showExpect(res.data, res).toHaveProperty("result");
        showExpect(res.data.result, res).toHaveLength(3);
        showExpect(res.data.result[2], res).toHaveProperty(
            "content",
            "public content"
        );
        showExpect(res.data.result[1], res).toHaveProperty(
            "content",
            "unlisted content"
        );
        showExpect(res.data.result[0], res).toHaveProperty(
            "content",
            "private content @user4"
        );
    });
});

describe("timeline", () => {
    let userToken1: string,
        userToken2: string,
        userToken3: string,
        userToken4: string;
    beforeAll(async () => {
        await truncateDB();
        [userToken1, userToken2, userToken3, userToken4] = await Promise.all([
            createAndLoginUser("user1", GOOD_PASSWORD),
            createAndLoginUser("user2", GOOD_PASSWORD),
            createAndLoginUser("user3", GOOD_PASSWORD),
            createAndLoginUser("user4", GOOD_PASSWORD),
        ]);

        // user3 follows user1
        const res = await axios.put(
            "/user/@user1/follow",
            {},
            authHeader(userToken3)
        );
        showExpect(res.status, res).toBe(200);

        const publicPost = await axios.post(
            "/post",
            {
                content: "public content",
                privacy: "public",
            },
            authHeader(userToken1)
        );
        showExpect(publicPost.status, res).toBe(200);

        const unlistedPost = await axios.post(
            "/post",
            {
                content: "unlisted content",
                privacy: "unlisted",
            },
            authHeader(userToken1)
        );
        showExpect(unlistedPost.status, res).toBe(200);

        const followerOnlyPost = await axios.post(
            "/post",
            {
                content: "follower content",
                privacy: "follower",
            },
            authHeader(userToken1)
        );
        showExpect(followerOnlyPost.status, res).toBe(200);

        const privatePost = await axios.post(
            "/post",
            {
                content: "private content @user4",
                privacy: "private",
            },
            authHeader(userToken1)
        );
        showExpect(privatePost.status, res).toBe(200);
    }, 60000);
    it("poster can see their own posts", async () => {
        const res = await axios.get("/timeline", authHeader(userToken1));
        showExpect(res.status, res).toBe(200);
        showExpect(res.data, res).toHaveProperty("result");
        showExpect(res.data.result, res).toHaveLength(4);
        showExpect(res.data.result[3], res).toHaveProperty(
            "content",
            "public content"
        );
        showExpect(res.data.result[2], res).toHaveProperty(
            "content",
            "unlisted content"
        );
        showExpect(res.data.result[1], res).toHaveProperty(
            "content",
            "follower content"
        );
        showExpect(res.data.result[0], res).toHaveProperty(
            "content",
            "private content @user4"
        );
    });
    it("non-follower see nothing", async () => {
        const res = await axios.get("/timeline", authHeader(userToken2));
        showExpect(res.status, res).toBe(200);
        showExpect(res.data, res).toHaveProperty("result");
        showExpect(res.data.result, res).toHaveLength(0);
    });
    it("follower can see follower post", async () => {
        const res = await axios.get("/timeline", authHeader(userToken3));
        showExpect(res.status, res).toBe(200);
        showExpect(res.data, res).toHaveProperty("result");
        showExpect(res.data.result, res).toHaveLength(3);
        showExpect(res.data.result[2], res).toHaveProperty(
            "content",
            "public content"
        );
        showExpect(res.data.result[1], res).toHaveProperty(
            "content",
            "unlisted content"
        );
        showExpect(res.data.result[0], res).toHaveProperty(
            "content",
            "follower content"
        );
    });
    it("mentioned user can see private post", async () => {
        const res = await axios.get("/timeline", authHeader(userToken4));
        showExpect(res.status, res).toBe(200);
        showExpect(res.data, res).toHaveProperty("result");
        showExpect(res.data.result, res).toHaveLength(1);
        showExpect(res.data.result[0], res).toHaveProperty(
            "content",
            "private content @user4"
        );
    });
});

describe("favorite and bookmark", () => {
    let userToken1: string;
    let userToken2: string;
    let postId: string;
    beforeAll(async () => {
        await truncateDB();
        userToken1 = await createAndLoginUser("admin", GOOD_PASSWORD);
        userToken2 = await createAndLoginUser("user2", GOOD_PASSWORD);

        const publicPost = await axios.post(
            "/post",
            {
                content: "public content",
                privacy: "public",
            },
            authHeader(userToken1)
        );
        showExpect(publicPost.status, publicPost).toBe(200);
        postId = publicPost.data.post_id;
    }, 60000);
    let favoriteSuccess = false;
    let bookmarkSuccess = false;
    let reactionSuccess = false;
    it("can favorite a public post", async () => {
        const res = await axios.put(
            "/post/" + postId + "/favorite",
            {},
            {
                ...authHeader(userToken2),
            }
        );
        showExpect(res.status, res).toBe(200);
        favoriteSuccess = true;
    });
    it("can bookmark a public post", async () => {
        const res = await axios.put(
            "/post/" + postId + "/bookmark",
            {},
            {
                ...authHeader(userToken2),
            }
        );
        showExpect(res.status, res).toBe(200);
        bookmarkSuccess = true;
    });
    it("can create a reaction", async () => {
        const res = await axios.post(
            "/post/" + postId + "/reaction",
            {
                reaction: "🎉",
                add: true,
            },
            {
                ...authHeader(userToken2),
            }
        );
        showExpect(res.status, res).toBe(200);
        reactionSuccess = true;
    });
    it("can delete a favorite", async () => {
        if (!favoriteSuccess) {
            expect().fail();
        }
        const res = await axios.delete("/post/" + postId + "/favorite", {
            ...authHeader(userToken2),
        });
        showExpect(res.status, res).toBe(200);
    });
    it("can delete a bookmark", async () => {
        if (!bookmarkSuccess) {
            expect().fail();
        }
        const res = await axios.delete("/post/" + postId + "/bookmark", {
            ...authHeader(userToken2),
        });
        showExpect(res.status, res).toBe(200);
    });
    it("can delete a reaction", async () => {
        if (!reactionSuccess) {
            expect().fail();
        }
        const res = await axios.post(
            "/post/" + postId + "/reaction",
            {
                reaction: "🎉",
                add: false,
            },
            {
                ...authHeader(userToken2),
            }
        );
        showExpect(res.status, res).toBe(200);
    });
});
