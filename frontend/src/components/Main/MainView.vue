<script lang="ts" setup>
import MainAppShell from '@/components/Main/MainAppShell.vue'
import MainFeed from '@/components/Main/MainFeed.vue'
import MainHeader from '@/components/Main/MainHeader.vue'
import MainLeftMenu from '@/components/Main/MainLeftMenu.vue'
import MainRightMenu from '@/components/Main/MainRightMenu.vue'
import ProfileView from '@/components/Profile/ProfileView.vue'
import CreatePostView from '@/components/Main/CreatePostView.vue'
import TrendPostView from '@/components/Trend/TrendPostList.vue'

import axios from 'axios'
import { provide } from 'vue'
import { getLoginToken, getUsername } from '../../auth'
import { AUTH_AXIOS, CURRENT_USERNAME } from '../../consts'
import { BASE_URL } from '../../settings'

// axios setup
const authAxios = axios.create({
    baseURL: BASE_URL
})
authAxios.interceptors.request.use(config => {
    const token = getLoginToken()
    if (token) {
        config.headers.Authorization = `Bearer ${token}`
    }

    return config
})

const props = defineProps<{
    mode: 'feed' | 'profile' | 'trend-search'
}>()

const username = getUsername()!

provide(AUTH_AXIOS, authAxios)
provide(CURRENT_USERNAME, username)
</script>

<template>
    <MainAppShell>
        <template #header>
            <MainHeader />
        </template>
        <template #left-menu>
            <MainLeftMenu @create-post="handleToggleCreatePost" />
        </template>
        <template #right-menu>
            <MainRightMenu />
        </template>
        <template #create-post>
            <CreatePostView :showPostMenu="isCreatePostOpen" />
        </template>
        <template #feed>
            <MainFeed v-if="props.mode === 'feed'" />
            <ProfileView v-else-if="props.mode === 'profile'" />
            <TrendPostView v-else-if="props.mode === 'trend-search'" />
        </template>
    </MainAppShell>
</template>

<script lang="ts">
export default {
    name: 'MainView',
    data() {
        return {
            isCreatePostOpen: false
        }
    },
    methods: {
        // Toggle Create Post
        handleToggleCreatePost() {
            this.isCreatePostOpen = !this.isCreatePostOpen
        }
    }
}
</script>

<style scoped></style>
