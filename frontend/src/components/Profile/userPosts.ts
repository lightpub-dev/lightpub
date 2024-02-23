import { Ref, computed, inject, ref, watchEffect } from 'vue'
import { AUTH_AXIOS } from '../../consts.ts'
import { UserPostEntry } from '../UserPost/userpost.model.ts'

export interface UserPostsResponse {
    posts: UserPostEntry[]
    next: string
    previous: string
}

export function useUserPosts(userspec: Ref<string>) {
    const authAxios = inject(AUTH_AXIOS)!

    const posts = ref<UserPostsResponse | null>(null)
    const nextURL = ref<string | null>(null)
    const nextFetchCount = ref<number>(0)

    const fetchPosts = async (doNotPush?: boolean) => {
        try {
            posts.value = null
            const response = await authAxios.get(
                `/user/${userspec.value}/posts`
            )
            if (!doNotPush) {
                posts.value = response.data
            }
            nextURL.value = response.data.next
            nextFetchCount.value = 0
            return response.data
        } catch (e) {
            console.error(e)
        }
    }

    const fetchNext = async (doNotPush?: boolean) => {
        if (!nextURL.value) return
        if (!posts.value) return

        try {
            const response = await authAxios.get(nextURL.value!)
            if (!doNotPush) {
                posts.value!.posts.push(...response.data.results)
            }
            nextURL.value = response.data.next
            nextFetchCount.value = nextFetchCount.value + 1
            return response.data
        } catch (e) {
            console.error(e)
        }
    }

    const reloadPosts = async () => {
        try {
            const newPosts = await fetchPosts(true)
            for (let i = 0; i < nextFetchCount.value; i++) {
                newPosts.push(...(await fetchNext(true)))
            }
            posts.value = newPosts
        } catch (e) {
            console.error(e)
        }
    }

    const hasNext = computed(() => {
        const next = nextURL.value !== null
        return next
    })

    watchEffect(async () => {
        await fetchPosts()
    })

    return {
        posts,
        fetchPosts,
        fetchNext,
        reloadPosts,
        hasNext
    }
}
