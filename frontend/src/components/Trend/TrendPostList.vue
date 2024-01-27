<script lang="ts" setup>
import { Ref, computed, inject, ref, watchEffect } from 'vue'
import { useRoute } from 'vue-router'
import { AUTH_AXIOS } from '../../consts'
import UserPost from '@/components/UserPost/UserPost.vue'
import { UserPostEntry } from '../UserPost/userpost.model'

const route = useRoute()
const hashtag = computed(() => route.query.hashtag as string)
const axios = inject(AUTH_AXIOS)!

const posts: Ref<UserPostEntry[]> = ref([])
watchEffect(async () => {
    posts.value = []
    const { data } = await axios.get<{ results: UserPostEntry[] }>(`/posts`, {
        params: {
            hashtag: hashtag.value
        }
    })
    posts.value = data.results
})
</script>

<template>
    <div
        class="grid-cols-1 w-full grid md:grid-cols-1 px-20 pt-5 transition-all bg-gray-100"
    >
        <div class="text-left text-lg">
            <span class="text-gray-500">#</span>
            <span class="text-black">{{ hashtag }}</span>
        </div>
        <div class="flex flex-col p-2">
            <UserPost
                v-for="(post, index) in posts"
                :key="index"
                :user_post="post"
            ></UserPost>
        </div>
    </div>
</template>
