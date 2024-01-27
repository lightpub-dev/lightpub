import { computed, inject, ref, watchEffect } from 'vue'
import { AUTH_AXIOS } from '../../consts.ts'
import { TimelineResponse } from './userpost.model'

export function useTimeline() {
    const authAxios = inject(AUTH_AXIOS)!

    const posts = ref<TimelineResponse | null>(null)
    const nextURL = ref<string | null>(null)

    const fetchPosts = async () => {
        try {
            const response = await authAxios.get('/timeline')
            posts.value = response.data
            nextURL.value = response.data.next
        } catch (e) {
            console.error(e)
        }
    }

    const fetchNext = async () => {
        if (!nextURL.value) return
        if (!posts.value) return

        try {
            const response = await authAxios.get(nextURL.value!)
            posts.value!.results.push(...response.data.results)
            nextURL.value = response.data.next
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
        hasNext
    }
}
