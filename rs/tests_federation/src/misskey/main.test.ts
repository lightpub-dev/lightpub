import { describe, test, before } from "mocha";
import expect from "expect.js";
import axios from "axios";

const MISSKEY_BASE_URL = "https://misskey.tinax.local";
const MISSKEY_TOKEN = "p6jFYxtFO7QJGbQSpZthKJzTPGfzqhQt";
const MISSKEY_USER_ID = "9r70xhde0mav0001";
const LP_BASE_URL = "https://lightpub.tinax.local";

async function sleep(ms: number) {
    return new Promise((resolve) => {
        setTimeout(resolve, ms);
    });
}

describe("Misskey federation test", function () {
    let lpToken: string;

    before(async function () {
        this.timeout(0);
        // create admin user for lightpub
        const res = await axios.post(LP_BASE_URL + "/register", {
            username: "admin",
            password: "1234abcd",
            nickname: "admin dayo",
        });
        expect(res.status).to.be(200);
        // login
        const res2 = await axios.post(LP_BASE_URL + "/login", {
            username: "admin",
            password: "1234abcd",
        });
        expect(res2.status).to.be(200);
        lpToken = res2.data.token;
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

    context("user follow test", function () {
        this.timeout(0);
        let followSuccess = false;
        let lightpubUserId: string;
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
            });
        });
    });
});
