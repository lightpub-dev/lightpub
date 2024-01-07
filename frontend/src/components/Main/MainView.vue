<script lang="ts" setup>
import MainAppShell from '@/components/Main/MainAppShell.vue'
import MainFeed from '@/components/Main/MainFeed.vue'
import MainHeader from '@/components/Main/MainHeader.vue'
import MainLeftMenu from '@/components/Main/MainLeftMenu.vue'
import MainRightMenu from '@/components/Main/MainRightMenu.vue'
import CreatePostView from '@/components/Main/CreatePostView.vue'

import axios from 'axios'
import { provide } from 'vue'
import { getLoginToken } from '../../auth'
import { AUTH_AXIOS } from '../../consts'
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

provide(AUTH_AXIOS, authAxios)
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
            <MainFeed />
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
