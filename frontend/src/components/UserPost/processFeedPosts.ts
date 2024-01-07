import { inject, ref, watchEffect } from 'vue'
import { AUTH_AXIOS } from '../../consts.ts'
import { TimelineResponse } from './post.ts'

export function useTimeline() {
    const authAxios = inject(AUTH_AXIOS)!

    const posts = ref<TimelineResponse | null>(null)

    watchEffect(async () => {
        try {
            const response = await authAxios.get('/timeline')
            posts.value = response.data
        } catch (e) {
            console.error(e)
        }
    })

    return {
        posts
    }
}
