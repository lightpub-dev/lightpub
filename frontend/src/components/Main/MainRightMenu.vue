<script lang="ts" setup>
import { inject, ref, watchEffect } from 'vue'
import { AUTH_AXIOS } from '../../consts'

const authedAxios = inject(AUTH_AXIOS)!

const trendings = ref<
    Array<{
        index: number
        hashtag: string
        post_count: number
        url: string
    }>
>([])

watchEffect(async () => {
    try {
        const res = await authedAxios.get('/trend')
        for (let i = 0; i < res.data.trends.length; i++) {
            const trend = res.data.trends[i]
            trendings.value.push({
                index: i + 1,
                hashtag: trend.hashtag,
                post_count: trend.post_count,
                url: `/trend/posts?hashtag=${encodeURIComponent(trend.hashtag)}`
            })
        }
    } catch (err) {
        console.log(err)
        trendings.value = []
    }
})
</script>
<template>
    <div
        class="w-full h-full flex flex-col py-2 px-2 bg-gray-100 overflow-y-auto"
    >
        <ul class="w-full bg-green-100 p-5 rounded-lg shadow-xl mb-5">
            <li class="text-green-600 text-lg font-semibold">
                <p>Trends</p>
            </li>
            <div class="my-2 border-b border-green-600 w-full"></div>
            <router-link
                v-for="(trending, index) in trendings"
                :key="index"
                :to="trending.url"
            >
                <li
                    class="px-2 py-2 last:mb-0 hover:bg-green-300 rounded-md transition-colors duration-200 cursor-pointer"
                >
                    <!-- <p class="text-xs text-gray-500 dark:text-gray-400">
                    {{ index + 1 }} - {{ trending.gender }}
                </p> -->
                    <p class="text-lg text-gray-800 font-bold">
                        {{ trending.hashtag }}
                    </p>
                    <p class="text-xs text-gray-500 dark:text-gray-400">
                        {{ trending.post_count }} posts
                    </p>
                </li>
            </router-link>
        </ul>
    </div>
</template>

<script lang="ts">
export default {}
</script>
