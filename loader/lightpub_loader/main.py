import asyncio
import aiohttp
import random
import time
from dataclasses import dataclass
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


@dataclass
class User:
    username: str
    password: str
    user_id: str
    cookie_jar: aiohttp.CookieJar


def probability(prob: float) -> bool:
    return random.random() < prob


class LightpubLoadTest:
    def __init__(self, base_url: str, num_users: int):
        self.base_url = base_url
        self.num_users = num_users
        self.users: list[User] = []
        self.note_ids: list[str] = []
        self.hashtags = [f"tag{i}" for i in range(10)]

    async def generate_users(self):
        """Create N users and login"""
        for i in range(self.num_users):
            time_str = int(time.time())
            # strip time_str to last 8 digits to avoid username length limit
            time_str = str(time_str)[-8:]
            username = f"lt_user_{i}_{time_str}"
            password = "testpass123"
            cookie_jar = aiohttp.CookieJar()

            # Register user
            async with aiohttp.ClientSession(cookie_jar=cookie_jar) as session:
                async with session.post(
                    f"{self.base_url}/auth/register",
                    json={
                        "username": username,
                        "nickname": f"Load Test User {i}",
                        "password": password,
                    },
                ) as response:
                    if response.status != 200:
                        logger.error(f"Failed to register user {username}")
                        continue
                    user_id = (await response.json())["userId"]

                # Login
                async with session.post(
                    f"{self.base_url}/auth/login",
                    json={"username": username, "password": password},
                ) as response:
                    if response.status != 200:
                        logger.error(f"Failed to login user {username}")
                        continue

            self.users.append(User(username, password, user_id, cookie_jar))
            logger.info(f"Created user {username}")

    def generate_content(self) -> str:
        """Generate random content with mentions and hashtags"""
        content = f"Load test content {random.randint(1000, 9999)}"

        # Maybe add mention
        if probability(0.3) and self.users:  # 30% chance
            mentioned_user = random.choice(self.users)
            content += f" @{mentioned_user.username}"

        # Maybe add hashtag
        if probability(0.3):  # 30% chance
            hashtag = random.choice(self.hashtags)
            content += f" #{hashtag}"

        return content

    async def create_note(self, session: aiohttp.ClientSession) -> None:
        """Create a note with random content"""
        # Weighted visibility selection
        visibility_options = {
            "public": 0.6,
            "unlisted": 0.2,
            "follower": 0.1,
            "private": 0.1,
        }
        content_type_options = ["plain", "md"]

        # Select visibility based on weights
        visibility = random.choices(
            list(visibility_options.keys()), weights=list(visibility_options.values())
        )[0]

        mpwriter = aiohttp.MultipartWriter("form-data")

        # Add content field
        part = mpwriter.append(self.generate_content())
        part.headers.update({"Content-Disposition": 'form-data; name="content"'})

        # Add contentType field
        content_type = random.choice(content_type_options)
        part = mpwriter.append(content_type)
        part.headers.update({"Content-Disposition": 'form-data; name="contentType"'})

        # Add visibility field
        part = mpwriter.append(visibility)
        part.headers.update({"Content-Disposition": 'form-data; name="visibility"'})

        # Maybe make it a reply
        if self.note_ids and random.random() < 0.2:  # 20% chance
            reply_id = random.choice(self.note_ids)
            part = mpwriter.append(reply_id)
            part.headers.update({"Content-Disposition": 'form-data; name="replyToId"'})

        async with session.post(f"{self.base_url}/note", data=mpwriter) as response:
            if response.status == 200:
                response_data = await response.json()
                self.note_ids.append(response_data["noteId"])
                logger.debug("Created note")
            else:
                logger.error("Failed to create note")
                logger.error(await response.text())

    async def follow_user(self, self_user: str, session: aiohttp.ClientSession) -> None:
        """Follow a random user"""
        if len(self.users) < 2:
            return

        target_user = random.choice(self.users)
        if target_user.user_id == self_user.user_id:
            return
        async with session.post(
            f"{self.base_url}/user/{target_user.user_id}/interaction",
            json={"type": "follow"},
        ) as response:
            if response.status == 200:
                logger.debug(f"Followed user {target_user.username}")
            else:
                logger.error(f"Failed to follow user {target_user.username}")
                logger.error(await response.text())

    async def random_action(self) -> None:
        """Perform random action (create note or follow) with random user"""
        user = random.choice(self.users)

        async with aiohttp.ClientSession(cookie_jar=user.cookie_jar) as session:
            if probability(0.7):  # 70% chance to create note
                await self.create_note(session)
            else:  # 30% chance to follow
                await self.follow_user(user, session)

    async def run_load_test(self, duration_seconds: int = 30):
        """Run load test for specified duration"""
        logger.info("Starting load test")

        # Create users first
        await self.generate_users()
        if not self.users:
            logger.error("No users created, cannot continue")
            return

        # Track start time
        start_time = time.time()
        tasks = []

        # Keep creating new tasks until duration is reached
        while time.time() - start_time < duration_seconds:
            # Create multiple concurrent tasks
            for _ in range(self.num_users):  # concurrent tasks = num_users
                task = asyncio.create_task(self.random_action())
                tasks.append(task)

            # Wait for all tasks to complete
            await asyncio.gather(*tasks)
            tasks.clear()

        total_time = time.time() - start_time
        total_notes = len(self.note_ids)
        logger.info(
            f"Load test completed. Created {total_notes} notes in {total_time:.2f} seconds"
        )
        logger.info(f"Average rate: {total_notes / total_time:.2f} notes per second")


async def main():
    load_tester = LightpubLoadTest(base_url="http://localhost:8000", num_users=30)
    await load_tester.run_load_test(duration_seconds=30)


if __name__ == "__main__":
    asyncio.run(main())
