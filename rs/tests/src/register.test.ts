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
