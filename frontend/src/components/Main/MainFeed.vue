<script lang="ts" setup>
import { computed } from 'vue'
import { useTimeline } from '../Post/processFeedPosts.ts'
const timeline = useTimeline()
const feedPosts = computed(() => {
    if (timeline.posts.value === null) {
        return []
    }
    return timeline.posts.value.posts
})
</script>

<template>
    <div
        class="grid-cols-1 w-full grid md:grid-cols-1 px-20 pt-5 transition-all bg-gray-100"
    >
        <div class="flex flex-col p-2">
            <Post
                v-for="(post, index) in feedPosts"
                :key="index"
                :user_post="post"
            ></Post>
        </div>
    </div>
</template>

<script lang="ts">
import Post from '@/components/Post/Post.vue'

export default {
    components: { Post }
}
</script>
