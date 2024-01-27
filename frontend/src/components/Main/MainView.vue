<script lang="ts" setup>
import CreatePostView from '@/components/Main/CreatePostView.vue'
import MainAppShell from '@/components/Main/MainAppShell.vue'
import MainFeed from '@/components/Main/MainFeed.vue'
import MainHeader from '@/components/Main/MainHeader.vue'
import MainLeftMenu from '@/components/Main/MainLeftMenu.vue'
import MainRightMenu from '@/components/Main/MainRightMenu.vue'
import ProfileView from '@/components/Profile/ProfileView.vue'
import UserList from '@/components/Profile/UserList.vue'
import TrendPostView from '@/components/Trend/TrendPostList.vue'
import DetailedPost from '@/components/UserPost/DetailedPost.vue'
import PasswordChange from '@/components/Login/PasswordChange.vue'

import axios from 'axios'
import { provide, ref } from 'vue'
import { getLoginToken, getUsername } from '../../auth'
import { AUTH_AXIOS, CURRENT_USERNAME } from '../../consts'
import { BASE_URL } from '../../settings'
import { eventBus } from '../../event'

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
    mode:
        | 'feed'
        | 'profile'
        | 'trend-search'
        | 'followers'
        | 'followings'
        | 'post-detail'
        | 'change-password'
}>()

const username = getUsername()!

provide(AUTH_AXIOS, authAxios)
provide(CURRENT_USERNAME, username)

const isCreatePostOpen = ref(false)
const replyToId = ref<string | null>(null)

const handleToggleCreatePost = () => {
    isCreatePostOpen.value = !isCreatePostOpen.value
}
const onCancel = () => {
    replyToId.value = null
}
const onCreate = () => {
    replyToId.value = null
}

eventBus.on('create-reply', (id: string) => {
    replyToId.value = id
    isCreatePostOpen.value = true
})
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
            <CreatePostView
                :showPostMenu="isCreatePostOpen"
                :replyToId="replyToId"
                @created="onCreate"
                @canceled="onCancel"
            />
        </template>
        <template #feed>
            <MainFeed v-if="props.mode === 'feed'" />
            <ProfileView v-else-if="props.mode === 'profile'" />
            <TrendPostView v-else-if="props.mode === 'trend-search'" />
            <UserList v-else-if="props.mode === 'followers'" mode="followers" />
            <UserList
                v-else-if="props.mode === 'followings'"
                mode="followings"
            />
            <DetailedPost v-else-if="props.mode === 'post-detail'" />
            <PasswordChange v-else-if="props.mode === 'change-password'" />
        </template>
    </MainAppShell>
</template>

<script lang="ts">
export default {
    name: 'MainView'
}
</script>

<style scoped></style>
