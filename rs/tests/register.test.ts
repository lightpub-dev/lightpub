import axios from "axios";
import { test, describe, expect, beforeAll } from "bun:test";

const BASE_URL = "https://lightpub.tinax.local";

describe("/register", () => {
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
