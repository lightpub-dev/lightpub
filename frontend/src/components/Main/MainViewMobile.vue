<script lang="ts" setup>
import MainAppShellMobile from '@/components/Main/MainAppShellMobile.vue'
import MainLeftMenuModalMobile from './MainLeftMenuModalMobile.vue'
import MainRightMenuModalMobile from './MainRightMenuModalMobile.vue'
import MainHeaderMobile from './MainHeaderMobile.vue'

import CreatePostView from '@/components/Main/CreatePostView.vue'
import MainFeed from '@/components/Main/MainFeed.vue'
import ProfileView from '@/components/Profile/ProfileView.vue'
import UserList from '@/components/Profile/UserList.vue'
import TrendPostView from '@/components/Trend/TrendPostList.vue'
import DetailedPost from '@/components/UserPost/DetailedPost.vue'
import PasswordChange from '@/components/Login/PasswordChange.vue'
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
const isLeftMenuOpen = ref(false)
const isRightMenuOpen = ref(false)

const handleToggleCreatePost = () => {
    isCreatePostOpen.value = !isCreatePostOpen.value
}

const handleCreatePost = () => {
    isCreatePostOpen.value = true
}

const handleCreatePostClose = () => {
    isCreatePostOpen.value = false
}

const openLeftMenu = () => {
    isLeftMenuOpen.value = true
}

const closeLeftMenu = () => {
    isLeftMenuOpen.value = false
}

const openRightMenu = () => {
    isRightMenuOpen.value = true
}

const closeRightMenu = () => {
    isRightMenuOpen.value = false
}
</script>

<template>
    <MainAppShellMobile>
        <template #header>
            <MainHeaderMobile
                @on-menu-button-clicked="openLeftMenu"
                @on-trend-button-clicked="openRightMenu"
            />
        </template>

        <template #modals>
            <MainLeftMenuModalMobile
                :is-open="isLeftMenuOpen"
                @create-post="handleCreatePost"
                @on-close="closeLeftMenu"
            />
            <MainRightMenuModalMobile
                :is-open="isRightMenuOpen"
                @on-close="closeRightMenu"
            />
        </template>
        <template #create-post>
            <CreatePostView
                :showPostMenu="isCreatePostOpen"
                @on-close="handleCreatePostClose"
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
    </MainAppShellMobile>
</template>

<style scoped></style>
