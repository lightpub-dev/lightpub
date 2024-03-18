import axios from "axios";
import expect from "expect.js";
import { exec } from "child_process";
import mocha from "mocha";

const BASE_URL = "https://lightpub.tinax.local";

const truncateDB = async () => {
    // execute truncate_db.sh
    return new Promise((resolve, reject) => {
        const proc = exec("bash ./truncate_db.sh", (error, stdout, stderr) => {
            if (error) {
                console.error(`exec error: ${error}`);
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
        this.timeout(30000);
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
