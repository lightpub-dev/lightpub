import { By, Builder, Browser, WebDriver } from "selenium-webdriver";
import Firefox from "selenium-webdriver/firefox";
import { describe, test, before } from "mocha";
import expect from "expect.js";

const MISSKEY_BASE_URL = "https://misskey.tinax.local";

describe("Misskey federation test", function () {
    let driver: WebDriver;

    before(async function () {
        this.timeout(60000);
        const options = new Firefox.Options();
        options.setAcceptInsecureCerts(true);
        options.setPreference("intl.accept_languages", "ja-JP");
        driver = await new Builder()
            .forBrowser(Browser.FIREFOX)
            .setFirefoxOptions(options)
            .build();

        // login
        await driver.get(MISSKEY_BASE_URL);
        driver.manage().window().setSize(1000, 1000);
        await driver.sleep(5000);
        const loginButton = driver.findElement(
            By.css("button[data-cy-signin]"),
        );
        await loginButton.click();
        const username = driver.findElement(
            By.css('input[placeholder="ユーザー名"]'),
        );
        const password = driver.findElement(
            By.css('input[placeholder="パスワード"]'),
        );
        await username.sendKeys("missuser");
        await password.sendKeys("1234abcd");
        const submitButton = driver.findElement(By.css("button[type=submit]"));
        await submitButton.click();
    });

    describe("Misskey start check", async function () {
        before(async function () {
            this.timeout(5000);
            await driver.get(MISSKEY_BASE_URL);
        });

        test("success", function () {
            expect(true).to.be(true);
        });
    });

    after(async function () {
        if (driver) {
            await driver.quit();
        }
    });
});
