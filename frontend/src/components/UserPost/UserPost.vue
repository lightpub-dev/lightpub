<script lang="ts" setup>
import { format } from 'timeago.js'
import { PropType, computed, inject } from 'vue'
import { UserPostEntry } from './userpost.model.ts'
import { AUTH_AXIOS } from '../../consts'

const props = defineProps({
    user_post: {
        type: Object as PropType<UserPostEntry>,
        required: true
    }
})

const axios = inject(AUTH_AXIOS)!

const actualPost = computed(() => {
    if (
        props.user_post.repost_of !== undefined &&
        props.user_post.repost_of !== null &&
        typeof props.user_post.repost_of !== 'string'
    ) {
        return props.user_post.repost_of
    }
    return props.user_post
})

const content = computed(() => {
    return actualPost.value.content
})

const nickname = computed(() => {
    return actualPost.value.author.nickname
})

const username = computed(() => {
    return actualPost.value.author.username
})

const hostname = computed(() => {
    return actualPost.value.author.host
})

const atHostname = computed(() => {
    if (hostname.value === null) {
        return ''
    }
    return `@${hostname.value}`
})

const createdTime = computed(() => {
    return format(actualPost.value.created_at)
})

const replyCount = computed(() => {
    return actualPost.value.reply_count
})

const repostCount = computed(() => {
    return actualPost.value.repost_count + actualPost.value.quote_count
})

const favoriteCount = computed(() => {
    return actualPost.value.favorite_count
})

const userPageURL = computed(() => {
    const id = actualPost.value.author.id

    return `/user/${id}`
})

const isRepostedByMe = computed(() => {
    return actualPost.value.reposted_by_me ?? false
})

const onRepost = async () => {
    await axios.post(`/post/${props.user_post.id}/repost`, {
        privacy: 'public'
    })
}

const isFavoritedByMe = computed(() => {
    return actualPost.value.favorited_by_me ?? false
})

const onFavorite = async () => {
    if (isFavoritedByMe.value) {
        await axios.delete(`/post/${props.user_post.id}/favorite`)
    } else {
        await axios.put(`/post/${props.user_post.id}/favorite`)
    }
}

// const isBookmarkedByMe = computed(() => {
//     return actualPost.value.bookmarked_by_me ?? false
// })
</script>
<template>
    <div class="w-full p-5 bg-white rounded-md flex flex-col mb-4 rounded-xl">
        <!-- Add this div to display repost information -->
        <div
            v-if="
                props.user_post.repost_of !== undefined &&
                props.user_post.repost_of !== null
            "
            class="mb-2"
        >
            <p class="text-sm text-gray-500">
                Reposted by {{ props.user_post.author.nickname }}
            </p>
        </div>
        <div class="flex justify-between items-center">
            <div class="flex items-center">
                <!-- Avatar -->
                <router-link :to="userPageURL">
                    <div
                        class="avatar rounded-full bg-ll-base dark:bg-ld-base w-10 h-10 border-2 border-ll-border dark:border-ld-border mr-3 flex items-center justify-center"
                    >
                        <img
                            alt=""
                            class="h-full w-full rounded-full"
                            src="https://avatars.githubusercontent.com/u/41512077"
                        />
                    </div>
                </router-link>
                <!-- User Details -->
                <div class="flex items-center">
                    <router-link :to="userPageURL">
                        <p class="text-lg font-bold text-gray-800 mr-2">
                            {{ nickname }}
                        </p>
                    </router-link>
                    <router-link :to="userPageURL">
                        <p class="text-sm text-gray-800 mr-2">
                            @{{ username }}{{ atHostname }}
                        </p></router-link
                    >
                    <p class="text-sm text-gray-500">{{ createdTime }}</p>
                </div>
            </div>
            <button class="active:scale-95 transform transition-transform">
                <svg
                    class="w-6 h-6"
                    fill="currentColor"
                    viewBox="0 0 24 24"
                    xmlns="http://www.w3.org/2000/svg"
                >
                    <path
                        clip-rule="evenodd"
                        d="M4.5 12a1.5 1.5 0 113 0 1.5 1.5 0 01-3 0zm6 0a1.5 1.5 0 113 0 1.5 1.5 0 01-3 0zm6 0a1.5 1.5 0 113 0 1.5 1.5 0 01-3 0z"
                        fill-rule="evenodd"
                    />
                </svg>
            </button>
        </div>

        <p class="pt-5 text-gray-600 text-lg mb-4">
            {{ content }}
        </p>

        <!-- <div
            v-if="props.user_post?.post.pictures_url.length > 0"
            :class="`images w-full h-70 bg-ll-neutral dark:bg-ld-neutral rounded-xl my-4 overflow-hidden grid ${
                props.user_post?.post.pictures_url.length > 1
                    ? 'grid-cols-2'
                    : 'grid-cols-1'
            } gap-2`"
        >
            <div class="h-full">
                <img
                    :src="`${props.user_post?.post.pictures_url[0]}`"
                    alt=""
                    class="w-full h-70 object-cover"
                />
            </div>
            <div
                v-if="props.user_post?.post.pictures_url.length > 1"
                :class="`

            h-70 grid ${
                props.user_post?.post.pictures_url.length == 2
                    ? 'grid-cols-1 grid-rows-1'
                    : ''
            }
             ${
                 props.user_post?.post.pictures_url.length == 3
                     ? 'grid-cols-1 grid-rows-2'
                     : ''
             }
            ${
                props.user_post?.post.pictures_url.length == 4
                    ? 'grid-cols-2 grid-rows-2'
                    : ''
            }


            gap-2`"
            >
                <img
                    v-if="props.user_post?.post.pictures_url.length > 1"
                    :class="`w-full h-full   object-cover ${
                        props.user_post?.post.pictures_url.length == 3 &&
                        'row-span-1 col-span-1 h-full'
                    }`"
                    :src="`${props.user_post?.post.pictures_url[1]}`"
                    alt=""
                />
                <img
                    v-if="props.user_post?.post.pictures_url.length > 2"
                    :class="`w-full h-full   object-cover ${
                        props.user_post?.post.pictures_url.length == 3 &&
                        'row-span-2 col-span-1'
                    }`"
                    :src="`${props.user_post?.post.pictures_url[2]}`"
                    alt=""
                />
                <img
                    v-if="props.user_post?.post.pictures_url.length > 3"
                    :class="`w-full h-full   object-cover ${
                        props.user_post?.post.pictures_url.length == 4 &&
                        'col-span-2'
                    }`"
                    :src="`${props.user_post?.post.pictures_url[3]}`"
                    alt=""
                />
                <img
                    v-if="props.user_post?.post.pictures_url.length > 4"
                    :class="`w-full h-2/4   object-cover ${
                        props.user_post?.post.pictures_url.length == 5 &&
                        'col-span-3 row-span-1'
                    }`"
                    :src="`${props.user_post?.post.pictures_url[4]}`"
                    alt=""
                />
            </div>
        </div> -->

        <div
            class="flex justify-between pt-4 border-t border-ll-border dark:border-ld-border mt-4"
        >
            <button
                class="flex items-center active:scale-95 transform transition-transform"
            >
                <svg
                    class="w-6 h-6"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.5"
                    viewBox="0 0 24 24"
                    xmlns="http://www.w3.org/2000/svg"
                >
                    <path
                        d="M12 20.25c4.97 0 9-3.694 9-8.25s-4.03-8.25-9-8.25S3 7.444 3 12c0 2.104.859 4.023 2.273 5.48.432.447.74 1.04.586 1.641a4.483 4.483 0 01-.923 1.785A5.969 5.969 0 006 21c1.282 0 2.47-.402 3.445-1.087.81.22 1.668.337 2.555.337z"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    />
                </svg>
                <p class="ml-2">{{ replyCount }}</p>
            </button>
            <button
                class="flex items-center active:scale-95 transform transition-transform"
                @click="onRepost"
            >
                <svg
                    class="w-6 h-6"
                    fill="none"
                    :stroke="isRepostedByMe ? 'blue' : 'currentColor'"
                    stroke-width="1.5"
                    viewBox="0 0 24 24"
                    xmlns="http://www.w3.org/2000/svg"
                >
                    <path
                        d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182m0-4.991v4.99"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    />
                </svg>

                <p class="ml-2" :class="{ 'text-blue-500': isRepostedByMe }">
                    {{ repostCount }}
                </p>
            </button>
            <button
                class="flex items-center active:scale-95 transform transition-transform"
                @click="onFavorite"
            >
                <svg
                    class="w-6 h-6"
                    fill="none"
                    :stroke="isFavoritedByMe ? 'blue' : 'currentColor'"
                    stroke-width="1.5"
                    viewBox="0 0 24 24"
                    xmlns="http://www.w3.org/2000/svg"
                >
                    <path
                        d="M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12z"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    />
                </svg>

                <p class="ml-2" :class="{ 'text-blue-500': isFavoritedByMe }">
                    {{ favoriteCount }}
                </p>
            </button>
            <button
                class="flex items-center active:scale-95 transform transition-transform"
            >
                <svg
                    class="w-6 h-6"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.5"
                    viewBox="0 0 24 24"
                    xmlns="http://www.w3.org/2000/svg"
                >
                    <path
                        d="M9 8.25H7.5a2.25 2.25 0 00-2.25 2.25v9a2.25 2.25 0 002.25 2.25h9a2.25 2.25 0 002.25-2.25v-9a2.25 2.25 0 00-2.25-2.25H15m0-3l-3-3m0 0l-3 3m3-3V15"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    />
                </svg>
            </button>
        </div>
    </div>
</template>
<script lang="ts">
export default {}
</script>
<style lang=""></style>
