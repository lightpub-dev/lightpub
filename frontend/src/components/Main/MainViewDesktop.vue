<script lang="ts" setup>
import CreatePostView from '@/components/Main/CreatePostView.vue'
import MainFeed from '@/components/Main/MainFeed.vue'
import MainHeader from '@/components/Main/MainHeader.vue'
import MainLeftMenu from '@/components/Main/MainLeftMenu.vue'
import MainRightMenu from '@/components/Main/MainRightMenu.vue'
import ProfileView from '@/components/Profile/ProfileView.vue'
import UserList from '@/components/Profile/UserList.vue'
import TrendPostView from '@/components/Trend/TrendPostList.vue'
import DetailedPost from '@/components/UserPost/DetailedPost.vue'
import PasswordChange from '@/components/Login/PasswordChange.vue'

import MainAppShellDesktop from './MainAppShellDesktop.vue'
import { ref } from 'vue'

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

const isCreatePostOpen = ref(false)
const handleToggleCreatePost = () => {
    isCreatePostOpen.value = !isCreatePostOpen.value
}
</script>

<template>
    <MainAppShellDesktop>
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
            <UserList v-else-if="props.mode === 'followers'" mode="followers" />
            <UserList
                v-else-if="props.mode === 'followings'"
                mode="followings"
            />
            <DetailedPost v-else-if="props.mode === 'post-detail'" />
            <PasswordChange v-else-if="props.mode === 'change-password'" />
        </template>
    </MainAppShellDesktop>
</template>

<style scoped></style>
