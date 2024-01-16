<script lang="ts" setup>

import MainFeed from '@/components/Main/MainFeed.vue'
import MainLeftMenu from '@/components/Main/MainLeftMenu.vue'
import MainRightMenu from '@/components/Main/MainRightMenu.vue'
import ProfileView from '@/components/Profile/ProfileView.vue'
import CreatePostView from '@/components/Main/CreatePostView.vue'

import MainAppShellDesktop from './MainAppShellDesktop.vue'
import MainHeader from './MainHeader.vue'

const props = defineProps<{
    mode: 'feed' | 'profile',
}>()

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
        </template>
    </MainAppShellDesktop>
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
