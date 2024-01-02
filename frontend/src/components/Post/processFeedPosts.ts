import {UserPostEntry} from "./post.ts";

export const Posts: UserPostEntry[] = [
    {
        id: "0",
        author: {
            id: "das08",
            username: "Das08"
        },
        post: {
            text: "test",
            pictures_url: [
                "https://cdn.pixabay.com/photo/2017/03/21/09/51/car-2161701_1280.jpg",
            ],
            reactions: []
        },
        created_at: "2021-08-08T00:00:00.000Z",
        privacy: "public"
    },
    {
        id: "1",
        author: {
            id: "das08",
            username: "Das08"
        },
        post: {
            text: "I am a test post",
            pictures_url: [],
            reactions: []
        },
        created_at: "2021-08-08T00:00:00.000Z",
        privacy: "public"
    }
];


