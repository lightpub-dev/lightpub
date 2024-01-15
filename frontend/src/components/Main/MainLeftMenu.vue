<script lang="ts" setup>
import { computed, inject, ref } from 'vue'
import { AUTH_AXIOS, CURRENT_USERNAME } from '../../consts'

import { reactive } from 'vue'
import { useRouter } from 'vue-router'

const axios = inject(AUTH_AXIOS)!
const currentUsername = inject(CURRENT_USERNAME)!
const atUsername = computed(() => {
    if (currentUsername) {
        return `@${currentUsername}`
    } else {
        console.error('currentUsername is null')
        return ''
    }
})

const username = ref('')
const nickname = ref('')

const nPosts = ref(0)
const nFollowers = ref(0)
const nFollowings = ref(0)

const fetchProfile = async () => {
    const res = await axios.get(`/user/${atUsername.value}`)
    username.value = res.data.username
    nickname.value = res.data.nickname
    nPosts.value = res.data.counters.posts
    nFollowers.value = res.data.counters.followers
    nFollowings.value = res.data.counters.following
}

fetchProfile()

const userURL = computed(() => {
    if (username.value) {
        return `/user/${atUsername.value}`
    } else {
        return ''
    }
})

const router = useRouter()

const onFollowingJump = () => {
    router.push(`/user/${atUsername.value}/followings`)
}

const onFollowerJump = () => {
    router.push(`/user/${atUsername.value}/followers`)
}

const menus = reactive({
    active: 0,
    menusList: [
        {
            name: 'Home',
            icon: ['fa-solid', 'fa-house'],
            url: '/'
        },
        {
            name: 'Search',
            icon: ['fa-solid', 'fa-magnifying-glass']
        },
        {
            name: 'Notifications',
            icon: ['fa-solid', 'fa-bell']
        },
        {
            name: 'Favorite',
            icon: ['fa-solid', 'fa-heart']
        },
        {
            name: 'Direct Messages',
            icon: ['fa-solid', 'fa-inbox']
        },
        {
            name: 'Settings',
            icon: ['fa-solid', 'fa-gear']
        }
    ]
})
</script>
<template>
    <div
        class="w-full h-full flex flex-col p-10 px-5 relative overflow-y-auto overflow-x-hidden bg-blue-100"
    >
        <!-- Profile Section -->
        <div class="profile flex flex-col justify-center items-center">
            <router-link :to="userURL">
                <img
                    alt=""
                    class="inline-block h-20 w-20 rounded-full ring-2 ring-white"
                    src="https://avatars.githubusercontent.com/u/41512077"
                />
            </router-link>
            <div class="flex flex-col justify-center items-center">
                <router-link :to="userURL">
                    <p class="text-xl font-bold text-gray-800">
                        {{ nickname }}
                    </p></router-link
                >
                <router-link :to="userURL">
                    <p class="text-sm text-gray-700">
                        {{ atUsername }}
                    </p></router-link
                >
                <!-- <p class="text-sm text-gray-700">Joined May 2021</p> -->
            </div>
        </div>

        <!-- Stats Section -->
        <div
            class="w-full flex justify-between mt-5 pb-5 border-b border-gray-300"
        >
            <!-- Repeated Structure for Posts, Followers, Following -->
            <div class="flex flex-col justify-center items-center">
                <p class="-mt-1 text-xs text-gray-600">Posts</p>
                <p class="text-lg font-bold text-gray-800">{{ nPosts }}</p>
            </div>
            <div class="self-center bg-gray-300 w-px h-12"></div>
            <div class="flex flex-col justify-center items-center">
                <p
                    class="-mt-1 text-xs text-gray-600 cursor-pointer"
                    @click="onFollowerJump"
                >
                    Followers
                </p>
                <p
                    class="text-lg font-bold text-gray-800 cursor-pointer"
                    @click="onFollowerJump"
                >
                    {{ nFollowers }}
                </p>
            </div>
            <div class="self-center bg-gray-300 w-px h-12"></div>
            <div class="flex flex-col justify-center items-center">
                <p
                    class="-mt-1 text-xs text-gray-600 cursor-pointer"
                    @click="onFollowingJump"
                >
                    Following
                </p>
                <p
                    class="text-lg font-bold text-gray-800 cursor-pointer"
                    @click="onFollowingJump"
                >
                    {{ nFollowings }}
                </p>
            </div>
        </div>

        <!-- Menu Items -->
        <ul class="flex-grow flex flex-col w-full pt-5">
            <template v-for="menu in menus.menusList" :key="menu.name">
                <router-link v-if="menu.url" :to="menu.url"
                    ><li
                        class="w-full px-4 py-2 flex items-center mb-2 cursor-pointer hover:bg-blue-200 hover:rounded-xl select-none"
                    >
                        <font-awesome-icon
                            :icon="menu.icon"
                            class="text-blue-600"
                        />
                        <p class="ml-5 text-gray-800 font-medium">
                            {{ menu.name }}
                        </p>
                    </li>
                </router-link>
                <template v-else>
                    <li
                        class="w-full px-4 py-2 flex items-center mb-2 cursor-pointer hover:bg-blue-200 hover:rounded-xl select-none"
                    >
                        <font-awesome-icon
                            :icon="menu.icon"
                            class="text-blue-600"
                        />
                        <p class="ml-5 text-gray-800 font-medium">
                            {{ menu.name }}
                        </p>
                    </li>
                </template>
            </template>
        </ul>

        <!-- Post Button -->
        <button
            class="w-full py-3 px-4 flex items-center justify-center mb-2 cursor-pointer select-none bg-gradient-to-r from-blue-600 to-blue-500 hover:from-blue-700 hover:to-blue-600 rounded-xl shadow-md hover:shadow-lg transition duration-300 ease-in-out"
            @click="toggleCreatePost"
        >
            <font-awesome-icon
                :icon="['fa-solid', 'fa-plus']"
                class="text-white"
            />
            <p class="ml-3 text-white font-medium">Post</p>
        </button>
    </div>
</template>

<script lang="ts">
export default {
    methods: {
        toggleCreatePost() {
            this.$emit('create-post')
        }
    }
}
</script>
