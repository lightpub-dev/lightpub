<script lang="ts" setup>
import MainAppShellMobile from '@/components/Main/MainAppShellMobile.vue'

import MainFeed from '@/components/Main/MainFeed.vue'
import ProfileView from '@/components/Profile/ProfileView.vue'
import CreatePostView from '@/components/Main/CreatePostView.vue'
import MainHeaderMobile from './MainHeaderMobile.vue'
import MainLeftMenuModalMobile from './MainLeftMenuModalMobile.vue'
import MainRightMenuModalMobile from './MainRightMenuModalMobile.vue'

const props = defineProps<{
    mode: 'feed' | 'profile',
}>()

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
        </template>
    </MainAppShellMobile>
</template>

<script lang="ts">
export default {
    name: 'MainView',
    data() {
        return {
            isCreatePostOpen: false,
            isLeftMenuOpen: false,
            isRightMenuOpen: false,
        }
    },
    methods: {
        // Toggle Create Post
        handleToggleCreatePost() {
            this.isCreatePostOpen = !this.isCreatePostOpen;
        },

        handleCreatePost() {
            this.isCreatePostOpen = true;
        },

        handleCreatePostClose() {
            this.isCreatePostOpen = false;
        },

        openLeftMenu() {
            console.log('openLeftMenu');
            this.isLeftMenuOpen = true;
        },

        closeLeftMenu() {
            this.isLeftMenuOpen = false;
        },

        openRightMenu() {
            this.isRightMenuOpen = true;
        },

        closeRightMenu() {
            this.isRightMenuOpen = false;
        },
    }
}
</script>

<style scoped></style>
