import { describe, test, before } from "mocha";
import expect from "expect.js";
import axios from "axios";

const MISSKEY_BASE_URL = "https://misskey.tinax.local";
const MISSKEY_TOKEN = "p6jFYxtFO7QJGbQSpZthKJzTPGfzqhQt";
const LP_BASE_URL = "https://lightpub.tinax.local";

describe("Misskey federation test", function () {
    let lpToken: string;

    before(async function () {
        this.timeout(15000);
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

    context("user follow test", function () {
        test("follow missuser from lightpub", async function () {
            const res = await axios.put(
                LP_BASE_URL + "/user/@missuser@misskey.tinax.local/follow",
                {},
                {
                    ...lightpubAuth(),
                },
            );
            expect(res.status).to.be(200);
        });
    });
});
