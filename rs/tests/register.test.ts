import axios from "axios";
import { test, describe, expect, beforeEach } from "bun:test";
import { exec } from "child_process";

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

describe("/register", () => {
    beforeEach(async () => {
        await truncateDB();
    });
    test(
        "registering a new user",
        async () => {
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
            expect(response.status).toBe(200);
            expect(response.data).toHaveProperty("user_id");
        },
        {
            timeout: 30000,
        }
    );
    test(
        "registering duplicate users",
        async () => {
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
                expect(response.status).toBe(200);
                expect(response.data).toHaveProperty("user_id");
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
                expect(response.status).toBe(400);
            }
        },
        {
            timeout: 60000,
        }
    );
});
