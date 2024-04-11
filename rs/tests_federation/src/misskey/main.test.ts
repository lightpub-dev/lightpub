import { describe, test, before } from "mocha";
import expect from "expect.js";
import axios from "axios";

const MISSKEY_BASE_URL = "https://misskey.tinax.local";
const MISSKEY_TOKEN = "p6jFYxtFO7QJGbQSpZthKJzTPGfzqhQt";
const MISSKEY_USER_ID = "9r70xhde0mav0001";
const LP_BASE_URL = "https://lightpub.tinax.local";

const GOOD_PASSWORD = "1234AbcD!?";

async function sleep(ms: number) {
    return new Promise((resolve) => {
        setTimeout(resolve, ms);
    });
}

describe("Misskey federation test", function () {
    let lpToken: string;
    // let lpNoFollowToken: string;

    async function createLpUser(
        username: string,
        nickname: string,
        password: string,
    ): Promise<string> {
        // create admin user for lightpub
        const res = await axios.post(LP_BASE_URL + "/register", {
            username: "admin",
            password: GOOD_PASSWORD,
            nickname: "admin dayo",
        });
        expect(res.status).to.be(200);
        // login
        const res2 = await axios.post(LP_BASE_URL + "/login", {
            username: "admin",
            password: GOOD_PASSWORD,
        });
        expect(res2.status).to.be(200);
        const lpToken = res2.data.token;
        return lpToken;
    }

    before(async function () {
        this.timeout(0);
        // create admin user for lightpub
        lpToken = await createLpUser("admin", "admin dayo", GOOD_PASSWORD);
        // lpNoFollowToken = await createLpUser(
        //     "no_follow",
        //     "I have no followers",
        //     GOOD_PASSWORD,
        // );
    });

    function lightpubAuth() {
        return {
            headers: {
                Authorization: `Bearer ${lpToken}`,
            },
        };
    }

    function misskeyAuth() {
        return {
            headers: {
                Authorization: `Bearer ${MISSKEY_TOKEN}`,
            },
        };
    }

    let lightpubUserId: string = "";
    context("user follow test", function () {
        this.timeout(0);
        let followSuccess = false;
        let followedSuccess = false;
        context("follow missuser from lightpub", function () {
            let success = false;
            test("follow missuser from lightpub", async function () {
                this.timeout(10000);
                const res = await axios.put(
                    LP_BASE_URL + "/user/@missuser@misskey.tinax.local/follow",
                    {},
                    {
                        ...lightpubAuth(),
                    },
                );
                expect(res.status).to.be(200);
                success = true;

                // wait for follow request to be processed
                await sleep(5000);
            });
            test("check misskey followers", async function () {
                if (!success) this.skip();

                const res = await axios.post(
                    MISSKEY_BASE_URL + "/api/users/followers",
                    {
                        userId: MISSKEY_USER_ID,
                    },
                    {
                        ...misskeyAuth(),
                    },
                );
                expect(res.status).to.be(200);
                expect(res.data).to.have.length(1);
                expect(res.data[0].follower.username).to.equal("admin");
                followSuccess = true;
                lightpubUserId = res.data[0].follower.id;
            });
        });

        context("follow lightpub admin from missuser", function () {
            before(function () {
                // if lightpub user does not appear in misskey followers, we cannot know the userId of lightpub admin
                if (!followSuccess) this.skip();
            });

            let success = false;
            test("follow lightpub admin from missuser", async function () {
                this.timeout(10000);
                const res = await axios.post(
                    MISSKEY_BASE_URL + "/api/following/create",
                    {
                        userId: lightpubUserId,
                    },
                    {
                        ...misskeyAuth(),
                    },
                );
                expect(res.status).to.be(200);
                success = true;
                await sleep(5000);
            });
            test("check lightpub followers", async function () {
                if (!success) this.skip();
                const res = await axios.get(
                    LP_BASE_URL + "/user/@admin/followers",
                    {
                        ...lightpubAuth(),
                    },
                );
                expect(res.status).to.be(200);
                expect(res.data.result).to.have.length(1);
                expect(res.data.result[0].username).to.equal("missuser");
                followedSuccess = true;
            });
        });

        context("unfollow missuser from lightpub", function () {
            before(function () {
                if (!followSuccess) this.skip();
            });

            let unfollowSuccess = false;
            test("send unfollow request to lightpub", async function () {
                this.timeout(10000);
                const res = await axios.delete(
                    LP_BASE_URL + "/user/@missuser@misskey.tinax.local/follow",
                    {
                        ...lightpubAuth(),
                    },
                );
                expect(res.status).to.be(200);
                await sleep(5000);
                unfollowSuccess = true;
            });

            test("check misskey followers", async function () {
                if (!unfollowSuccess) this.skip();
                const res = await axios.post(
                    MISSKEY_BASE_URL + "/api/users/followers",
                    {
                        userId: MISSKEY_USER_ID,
                    },
                    {
                        ...misskeyAuth(),
                    },
                );
                expect(res.status).to.be(200);
                expect(res.data).to.have.length(0);
            });
        });

        context("unfollow lightpub admin from missuser", function () {
            before(function () {
                if (!followedSuccess) this.skip();
            });

            test("send unfollow request to misskey", async function () {
                this.timeout(10000);
                const res = await axios.post(
                    MISSKEY_BASE_URL + "/api/following/delete",
                    {
                        userId: lightpubUserId,
                    },
                    {
                        ...misskeyAuth(),
                    },
                );
                expect(res.status).to.be(200);
                await sleep(5000);
            });

            test("check lightpub followers", async function () {
                const res = await axios.get(
                    LP_BASE_URL + "/user/@admin/followers",
                    {
                        ...lightpubAuth(),
                    },
                );
                expect(res.status).to.be(200);
                expect(res.data.result).to.have.length(0);
            });
        });
    });

    context("post test", function () {
        before(async function () {
            if (!lightpubUserId) this.skip();
            this.timeout(20000);
            // follow admin@lp from missuser
            // follow missuser from admin@lp
            const res = await axios.put(
                LP_BASE_URL + "/user/@missuser@misskey.tinax.local/follow",
                {},
                {
                    ...lightpubAuth(),
                },
            );
            expect(res.status).to.be(200);
            const res2 = await axios.post(
                MISSKEY_BASE_URL + "/api/following/create",
                {
                    userId: lightpubUserId,
                },
                {
                    ...misskeyAuth(),
                },
            );
            expect(res2.status).to.be(200);
            await sleep(5000);
        });

        test("post from lightpub user is empty", async function () {
            const res = await axios.post(
                MISSKEY_BASE_URL + "/api/users/notes",
                {
                    userId: lightpubUserId,
                },
                {
                    ...misskeyAuth(),
                },
            );
            expect(res.status).to.be(200);
            expect(res.data).to.have.length(0);
        });

        context("create a public post from lightpub", function () {
            let postCreated = false;
            it("can create a public post on lightpub", async function () {
                this.timeout(5000);
                const res = await axios.post(
                    LP_BASE_URL + "/post",
                    {
                        content: "this is a public post #1",
                        privacy: "public",
                    },
                    {
                        ...lightpubAuth(),
                    },
                );
                expect(res.status).to.be(200);
                await sleep(3000);
                postCreated = true;
            });
            it("can see the public post on misskey", async function () {
                if (!postCreated) this.skip();
                const res = await axios.post(
                    MISSKEY_BASE_URL + "/api/users/notes",
                    {
                        userId: lightpubUserId,
                    },
                    {
                        ...misskeyAuth(),
                    },
                );
                expect(res.status).to.be(200);
                expect(res.data.length).to.be.greaterThan(0);
                let found = false;
                for (let i = 0; i < res.data.length; i++) {
                    if (res.data[i].text === "this is a public post #1") {
                        found = true;
                        break;
                    }
                }
                expect(found).to.be(true);
            });
        });

        context("create a unlisted post from lightpub", function () {
            let postCreated = false;
            it("can create a unlisted post on lightpub", async function () {
                this.timeout(5000);
                const res = await axios.post(
                    LP_BASE_URL + "/post",
                    {
                        content: "this is a unlisted post #1",
                        privacy: "unlisted",
                    },
                    {
                        ...lightpubAuth(),
                    },
                );
                expect(res.status).to.be(200);
                await sleep(3000);
                postCreated = true;
            });
            it("can see the unlisted post on misskey", async function () {
                if (!postCreated) this.skip();
                const res = await axios.post(
                    MISSKEY_BASE_URL + "/api/users/notes",
                    {
                        userId: lightpubUserId,
                    },
                    {
                        ...misskeyAuth(),
                    },
                );
                expect(res.status).to.be(200);
                expect(res.data.length).to.be.greaterThan(0);
                let found = false;
                for (let i = 0; i < res.data.length; i++) {
                    if (res.data[i].text === "this is a unlisted post #1") {
                        found = true;
                        break;
                    }
                }
                expect(found).to.be(true);
            });
        });

        context("create a follower-only post from lightpub", function () {
            let postCreated = false;
            it("can create a follower-only post on lightpub", async function () {
                this.timeout(5000);
                const res = await axios.post(
                    LP_BASE_URL + "/post",
                    {
                        content: "this is a follower post #1",
                        privacy: "follower",
                    },
                    {
                        ...lightpubAuth(),
                    },
                );
                expect(res.status).to.be(200);
                await sleep(3000);
                postCreated = true;
            });
            it("followers can see the follower-only post on misskey", async function () {
                if (!postCreated) this.skip();
                const res = await axios.post(
                    MISSKEY_BASE_URL + "/api/users/notes",
                    {
                        userId: lightpubUserId,
                    },
                    {
                        ...misskeyAuth(),
                    },
                );
                expect(res.status).to.be(200);
                expect(res.data.length).to.be.greaterThan(0);
                let found = false;
                for (let i = 0; i < res.data.length; i++) {
                    if (res.data[i].text === "this is a follower post #1") {
                        found = true;
                        break;
                    }
                }
                expect(found).to.be(true);
            });
        });

        context("create a private post from lightpub", function () {
            context("not mentioned", function () {
                let postCreated = false;
                it("can create a private post on lightpub", async function () {
                    this.timeout(5000);
                    const res = await axios.post(
                        LP_BASE_URL + "/post",
                        {
                            content: "this is a private post #1",
                            privacy: "private",
                        },
                        {
                            ...lightpubAuth(),
                        },
                    );
                    expect(res.status).to.be(200);
                    await sleep(3000);
                    postCreated = true;
                });
                it("cannot see the private post on misskey", async function () {
                    if (!postCreated) this.skip();
                    const res = await axios.post(
                        MISSKEY_BASE_URL + "/api/users/notes",
                        {
                            userId: lightpubUserId,
                        },
                        {
                            ...misskeyAuth(),
                        },
                    );
                    expect(res.status).to.be(200);
                    let found = false;
                    for (let i = 0; i < res.data.length; i++) {
                        if (res.data[i].text === "this is a private post #1") {
                            found = true;
                            break;
                        }
                    }
                    expect(found).to.be(false);
                });
            });
            context("mentioned", function () {
                let postCreated = false;
                it("can create a private post on lightpub", async function () {
                    this.timeout(5000);
                    const res = await axios.post(
                        LP_BASE_URL + "/post",
                        {
                            content:
                                "this is a private post #2 @missuser@misskey.tinax.local",
                            privacy: "private",
                        },
                        {
                            ...lightpubAuth(),
                        },
                    );
                    expect(res.status).to.be(200);
                    await sleep(3000);
                    postCreated = true;
                });
                it("mentioned user can see the private post on misskey", async function () {
                    if (!postCreated) this.skip();
                    const res = await axios.post(
                        MISSKEY_BASE_URL + "/api/users/notes",
                        {
                            userId: lightpubUserId,
                        },
                        {
                            ...misskeyAuth(),
                        },
                    );
                    expect(res.status).to.be(200);
                    expect(res.data.length).to.be.greaterThan(0);
                    let found = false;
                    for (let i = 0; i < res.data.length; i++) {
                        if (
                            res.data[i].text.includes(
                                "this is a private post #2",
                            )
                        ) {
                            found = true;
                            break;
                        }
                    }
                    expect(found).to.be(true);
                });
            });
        });
    });
});
