<script lang="ts" setup>
import { computed } from 'vue'
import { useTimeline } from '../UserPost/processFeedPosts.ts'
const timeline = useTimeline()
const feedPosts = computed(() => {
    if (timeline.posts.value === null) {
        return []
    }
    return timeline.posts.value.results
})

eventBus.on('post-created', async () => {
    await timeline.fetchPosts();
})
</script>

<template>
    <div
        class="grid-cols-1 w-full grid md:grid-cols-1 px-20 pt-5 transition-all bg-gray-100"
    >
        <div class="flex flex-col p-2">
            <UserPost
                v-for="(post, index) in feedPosts"
                :key="index"
                :user_post="post"
            ></UserPost>
        </div>
    </div>
</template>

<script lang="ts">
import UserPost from '@/components/UserPost/UserPost.vue'
import { eventBus } from '../../event';

export default {
    components: { UserPost }
}
</script>
