import { Ref, inject, ref, watchEffect } from 'vue'
import { AUTH_AXIOS } from '../../consts.ts'
import { UserPostEntry } from '../Post//post.ts'

export interface UserPostsResponse {
    posts: UserPostEntry[]
}

export function useUserPosts(userspec: Ref<string>) {
    const authAxios = inject(AUTH_AXIOS)!

    const posts = ref<UserPostsResponse | null>(null)

    watchEffect(async () => {
        try {
            posts.value = null
            const response = await authAxios.get(
                `/user/${userspec.value}/posts`
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
