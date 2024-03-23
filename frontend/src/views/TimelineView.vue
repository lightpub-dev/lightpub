<script setup lang="ts">
import type { UserPost } from '@/components/post/UserPost.vue'
import UserPostView from '@/components/post/UserPost.vue'
import { emitter } from '@/event'
import { axiosOptions } from '@/store'
import axios from 'axios'
import { onUnmounted, ref, watchEffect } from 'vue'

const posts = ref<UserPost[]>([])

async function reloadTimeline() {
  // fetch timeline
  const res = await axios.get('/timeline', {
    ...axiosOptions()
  })
  const result = res.data.result as Array<{
    id: string
    author: { id: string; uri: string; host: string | null; username: string; nickname: string }
    content: string
    repost_of_id: string | null
    reply_to_id: string | null
    created_at: string
    privacy: 'public' | 'unlisted' | 'follower' | 'private'
    counts: {
      replies: number
      reposts: number
      favorites: number
      reactions: Record<string, number>
    }
    reposted_by_you: boolean
    favorited_by_you: boolean
    bookmarked_by_you: boolean
  }>
  posts.value = result.map((p) => {
    return {
      id: p.id,
      content: p.content,
      author: {
        username: p.author.username,
        nickname: p.author.nickname,
        host: p.author.host ?? undefined
      },
      createdAt: new Date(p.created_at)
    }
  })
}

// Fetch posts from the server
watchEffect(async () => {
  await reloadTimeline()
})

// reload timeline when a new post is created
const postCreateHandler = async () => {
  await reloadTimeline()
}
emitter.on('newPostCreated', postCreateHandler)
onUnmounted(() => {
  emitter.off('newPostCreated', postCreateHandler)
})
</script>

<template>
  <div :key="p.id" v-for="p in posts">
    <UserPostView
      :nickname="p.author.nickname"
      :username="p.author.username"
      :host="p.author.host"
      :created-at="p.createdAt"
      :content="p.content"
    ></UserPostView>
  </div>
</template>
