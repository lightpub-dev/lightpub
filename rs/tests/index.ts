import axios from "axios";
import { assert, expect } from "chai";
import { describe } from "mocha";
 
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