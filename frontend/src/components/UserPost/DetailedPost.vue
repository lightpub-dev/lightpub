<script setup lang="ts">
import { computed, inject, ref, watchEffect } from 'vue'
import { useRoute } from 'vue-router'
import { AUTH_AXIOS } from '../../consts'
import { UserPostEntry } from './userpost.model'
import UserPost from './UserPost.vue'

const route = useRoute()
const postId = computed(() => route.params.id as string)
const axios = inject(AUTH_AXIOS)!

const posts = ref<UserPostEntry[]>([])

const fetchPost = async (id: string) => {
    const { data } = await axios.get<UserPostEntry>(`/posts/${id}`)
    posts.value.push(data)

    if (data.reply_to_id) {
        await fetchPost(data.reply_to_id)
    }
}

watchEffect(async () => {
    posts.value = []
    await fetchPost(postId.value)
})
</script>

<template>
    <div class="bg-white rounded-lg shadow-md p-4">
        <user-post v-for="post in posts" :key="post.id" :user_post="post" />
    </div>
</template>
