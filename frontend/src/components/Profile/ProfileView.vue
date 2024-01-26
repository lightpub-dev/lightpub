<script lang="ts" setup>
import { Ref, computed, inject, ref, watchEffect } from 'vue'
import { useRoute } from 'vue-router'
import { AUTH_AXIOS } from '../../consts'
import { useUserPosts } from './userPosts'
import UserPost from '@/components/UserPost/UserPost.vue'
import { getUsername } from '../../auth'

const route = useRoute()
const id = computed(() => route.params.id)
const axios = inject(AUTH_AXIOS)!

const username = ref('')
const nickname = ref('')
const hostname = ref('')
const bio = ref('')
const labels = ref<
    {
        key: string
        value: string
    }[]
>([])

const nPosts = ref(100)
const nFollowers = ref(50)
const nFollowings = ref(51)

const isFollowing = ref<boolean | null>(null)

const fetchProfile = async () => {
    const res = await axios.get(`/users/${id.value}`)
    username.value = res.data.username
    nickname.value = res.data.nickname
    hostname.value = res.data.hostname
    bio.value = res.data.bio
    labels.value = res.data.labels
    isFollowing.value = res.data.is_following
    nPosts.value = res.data.counters.posts
    nFollowers.value = res.data.counters.followers
    nFollowings.value = res.data.counters.following
}

watchEffect(() => {
    fetchProfile()
})

const atHostname = computed(() => {
    if (hostname.value) {
        return `@${hostname.value}`
    } else {
        return ''
    }
})

const userPosts = useUserPosts(id as Ref<string>)
const posts = computed(() => {
    if (userPosts.posts.value === null) {
        return []
    }
    return userPosts.posts.value.results
})

// follow button
const toggleFollow = async () => {
    if (isFollowing.value === null) {
        return
    }
    if (isFollowing.value) {
        await axios.delete(`/followings/${id.value}/`)
    } else {
        await axios.post("/followings/", {
            followee_spec: id.value
        })
    }
    isFollowing.value = null

    fetchProfile()
}

const followerLink = computed(() => `/followers?user=${id.value}`)
const followingLink = computed(() => `/followings?user=${id.value}`)

const isMe = computed(() => {
    const myUsername = getUsername()
    return myUsername === username.value
})
</script>

<template>
    <div class="bg-gray-100 flex items-start justify-center pt-10">
        <div class="bg-white p-6 rounded-lg shadow-lg w-64">
            <div class="mb-4">
                <img
                    class="w-20 h-20 mx-auto rounded-full"
                    src="https://placekitten.com/200/200"
                    alt="User icon"
                />
            </div>
            <div class="text-center">
                <div class="flex items-center justify-center mb-2">
                    <div>
                        <h2 class="text-xl font-semibold text-gray-700">
                            {{ nickname }}
                        </h2>
                        <h3 class="text-gray-500 mb-3">
                            @{{ username }}{{ atHostname }}
                        </h3>
                    </div>
                    <button
                        :class="{
                            'ml-2 px-2 py-1': true,
                            'bg-blue-500 text-white': isFollowing,
                            'bg-transparent text-blue-500 border border-blue-500':
                                !isFollowing
                        }"
                        @click="toggleFollow"
                        v-if="!isMe"
                    >
                        {{
                            isFollowing === null
                                ? 'Loading...'
                                : isFollowing
                                  ? 'Following'
                                  : 'Follow'
                        }}
                    </button>
                </div>

                <p class="text-gray-600">{{ bio }}</p>
            </div>

            <!-- Specification Table -->
            <div class="mt-4">
                <table class="w-full text-sm text-center text-gray-500">
                    <tbody>
                        <tr
                            class="bg-white border-b"
                            v-for="label in labels"
                            :key="label.key"
                        >
                            <th scope="row" class="px-2 py-1">
                                {{ label.key }}
                            </th>
                            <td class="px-2 py-1">{{ label.value }}</td>
                        </tr>
                    </tbody>
                </table>
            </div>
            <!-- Counter Section -->
            <div class="flex justify-between mt-4">
                <div class="text-center">
                    <p class="text-sm text-gray-600">Posts</p>
                    <p class="font-semibold text-gray-700">{{ nPosts }}</p>
                </div>
                <div class="text-center">
                    <router-link :to="followerLink">
                        <p class="text-sm text-gray-600">Followers</p>
                        <p class="font-semibold text-gray-700">
                            {{ nFollowers }}
                        </p>
                    </router-link>
                </div>
                <div class="text-center">
                    <router-link :to="followingLink"
                        ><p class="text-sm text-gray-600">Following</p>
                        <p class="font-semibold text-gray-700">
                            {{ nFollowings }}
                        </p>
                    </router-link>
                </div>
            </div>
        </div>
    </div>
    <div
        class="grid-cols-1 w-full grid md:grid-cols-1 px-20 pt-5 transition-all bg-gray-100"
    >
        <div class="flex flex-col p-2">
            <UserPost
                v-for="(post, index) in posts"
                :key="index"
                :user_post="post"
            ></UserPost>
        </div>
    </div>
</template>
