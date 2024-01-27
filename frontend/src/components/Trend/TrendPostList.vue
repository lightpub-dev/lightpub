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
const nextURL = ref<string | null>('')
const fetchNext = async () => {
    if (nextURL.value === null) {
        return
    }
    const params = {
        hashtag: hashtag.value
    } as any
    if (nextURL.value !== '' && nextURL.value !== null) {
        params.next = nextURL.value
    }
    const { data } = await axios.get<{
        results: UserPostEntry[]
        next: string | null
    }>(`/posts`, {
        params
    })
    posts.value = data.results
    nextURL.value = data.next
}
watchEffect(async () => {
    await fetchNext()
})

const divRef = ref<HTMLDivElement | null>(null)
let loadingNow = false
const scrolledToBottom = async (e: Event) => {
    if (loadingNow) {
        return
    }
    try {
        loadingNow = true
        await fetchNext()
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
