<script lang="ts" setup>
import { computed, ref, watchEffect } from 'vue'
import { useTimeline } from '../UserPost/processFeedPosts.ts'

const timeline = useTimeline()

const feedPosts = computed(() => {
    if (timeline.posts.value === null) {
        return []
    }
    return timeline.posts.value.results
})

eventBus.on('post-created', async () => {
    await timeline.fetchPosts()
})

const divRef = ref<HTMLDivElement | null>(null)

let loadingNow = false
const scrolledToBottom = async (e: Event) => {
    if (loadingNow) {
        return
    }
    try {
        loadingNow = true
        await timeline.fetchNext()
    } catch (ex) {
        console.error(ex)
        alert('Failed to fetch next posts')
    } finally {
        loadingNow = false
    }
}

watchEffect(() => {
    if (divRef.value) {
        const myDiv = divRef.value
        divRef.value.addEventListener('scroll', e => {
            // console.log('left hand', myDiv.offsetHeight + myDiv.scrollTop)
            // console.log('right hand', myDiv.scrollHeight)
            if (
                myDiv.offsetHeight + myDiv.scrollTop >=
                myDiv.scrollHeight * 0.8
            ) {
                scrolledToBottom(e)
            }
        })
    }
})
</script>

<template>
    <div
        class="grid-cols-1 w-full grid md:grid-cols-1 px-20 pt-5 transition-all bg-gray-100 overflow-y-auto"
        ref="divRef"
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
import { eventBus } from '../../event'

export default {
    components: { UserPost }
}
</script>
