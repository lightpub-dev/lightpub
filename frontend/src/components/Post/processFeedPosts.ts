import { inject, ref } from 'vue'
import { AUTH_AXIOS } from '../../consts.ts'
import { TimelineResponse, UserPostEntry } from './post.ts'

interface ExampleUserPostEntry extends UserPostEntry {
    post: {
        text: string
        pictures_url: string[]
        reactions: string[]
    }
}

export const Posts: ExampleUserPostEntry[] = [
    {
        id: '0',
        author: {
            id: 'das08',
            username: 'Das08',
            host: 'localhost:1323'
        },
        content: 'test',
        post: {
            text: 'test',
            pictures_url: [
                'https://cdn.pixabay.com/photo/2017/03/21/09/51/car-2161701_1280.jpg'
            ],
            reactions: []
        },
        created_at: '2021-08-08T00:00:00.000Z',
        privacy: 'public'
    },
    {
        id: '1',
        author: {
            id: 'das08',
            username: 'Das08',
            host: 'localhost:1323'
        },
        content: 'I am a test post',
        post: {
            text: 'I am a test post',
            pictures_url: [],
            reactions: []
        },
        created_at: '2021-08-08T00:00:00.000Z',
        privacy: 'public'
    }
]

export function useTimeline() {
    const authAxios = inject(AUTH_AXIOS)!

    const posts = ref<TimelineResponse | null>(null)

    const reloadTimeline = async () => {
        try {
            const response = await authAxios.get('/timeline')
            posts.value = response.data
        } catch (e) {
            console.error(e)
        }
    }

    reloadTimeline()

    return {
        posts,
        reloadTimeline
    }
}
