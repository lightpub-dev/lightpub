export function add(a: number, b: number): number {
  return a + b;
}

// Learn more at https://deno.land/manual/examples/module_metadata#concepts
if (import.meta.main) {
  console.log("Add 2 + 3 =", add(2, 3));
}

/*
import axios from "npm:axios";
import { assert, expect } from "npm:chai";
import { describe } from "npm:mocha";

describe("/register", () => {
    it("registering a new user", async () => {
        const response = await axios.post("localhost:8000/register", {
            username: "username",
            nickname: "nickname",
            password: "password",
        });
        assert.equal(response.status, 200);
        expect(response).to.have.property("user_id");
    });
});
*/