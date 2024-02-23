import { computed, inject, ref, watchEffect } from 'vue'
import { AUTH_AXIOS } from '../../consts.ts'
import { TimelineResponse } from './userpost.model'

export function useTimeline() {
    const authAxios = inject(AUTH_AXIOS)!

    const posts = ref<TimelineResponse | null>(null)
    const nextURL = ref<string | null>(null)
    const nextFetchCount = ref<number>(0)

    const fetchPosts = async (doNotPush?: boolean) => {
        try {
            const response = await authAxios.get('/timeline')
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

    watchEffect(async () => {
        await fetchPosts()
    })

    const hasNext = computed(() => {
        const next = nextURL.value !== null
        return next
    })

    return {
        posts,
        fetchPosts,
        fetchNext,
        reloadPosts,
        hasNext
    }
}
