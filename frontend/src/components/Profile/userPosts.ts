import { Ref, inject, ref, watchEffect } from 'vue'
import { AUTH_AXIOS } from '../../consts.ts'
import { UserPostEntry } from '../UserPost/userpost.model.ts'

export interface UserPostsResponse {
    results: UserPostEntry[]
    next: string
    previous: string
}

export function useUserPosts(userspec: Ref<string>) {
    const authAxios = inject(AUTH_AXIOS)!

    const posts = ref<UserPostsResponse | null>(null)

    watchEffect(async () => {
        try {
            posts.value = null
            const response = await authAxios.get(
                `/posts?user=${userspec.value}`
            )
            posts.value = response.data
        } catch (e) {
            console.error(e)
        }
    })

    return {
        posts
    }
}
