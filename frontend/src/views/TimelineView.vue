< script setup lang=" ts">
  import type { UserPost } from '@/components/post/UserPost.vue';
  import UserPost from '@/components/post/UserPost.vue';
  import { useAuthStore } from '@/store';
  import axios from 'axios';
  import { ref, watchEffect } from 'vue';

  const posts = ref
  <UserPost[]>([]);
  const authStore = useAuthStore();

  // Fetch posts from the server
  watchEffect(async () => {
  // fetch timeline
  const res = await axios.get("/timeline", {
  ...authStore.axiosOptions
  })
  const result = res.data.result as Array<{ id: string, author: { id: string, uri: string, host: string | null,
    username: string, nickname: string }, content: string, repost_of_id: string | null, reply_to_id: string |
    null, created_at: string, privacy: 'public' | 'unlisted' | 'follower' | 'private' , counts: { replies: number,
    reposts: number, favorites: number, reactions: Record<string, number>
    },
    reposted_by_you: boolean,
    favorited_by_you: boolean,
    bookmarked_by_you: boolean
    }>;
    posts.value = result.map((p) => {
    return {
    content: p.content,
    author: {
    username: p.author.username,
    nickname: p.author.nickname,
    host: p.author.host ?? undefined
    },
    createdAt: new Date(p.created_at)
    }
    })
    })
    </script>

<template>
  <div v-for="p in posts">
    <UserPost :nickname="p.author.nickname" :username="p.author.username" :host="p.author.host"></UserPost>
  </div>
</template>
