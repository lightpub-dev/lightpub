import axios from "axios";
import { test, describe, expect, beforeAll } from "bun:test";

describe("/register", () => {
    test("registering a new user", async () => {
        const response = await axios.post("http://localhost:8000/register", {
            username: "initialuser",
            nickname: "initialnick",
            password: "password",
        });
        expect(response.status).toBe(200);
        expect(response.data).toHaveProperty("user_id");
    });
    test("registering duplicate users", async () => {
        {
            const response = await axios.post("http://localhost:8000/register", {
                username: "duplicateduser",
                nickname: "duplicatednick",
                password: "password",
            });
            expect(response.status).toBe(200);
            expect(response.data).toHaveProperty("user_id");
        }
        {
            const response = await axios.post("http://localhost:8000/register", {
                username: "duplicateduser",
                nickname: "duplicatednick",
                password: "password",
            });
            expect(response.status).toBe(400);
        }
    });
});