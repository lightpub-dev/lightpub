<script lang="ts" setup>
import MainAppShell from '@/components/Main/MainAppShell.vue'
import MainHeader from '@/components/Main/MainHeader.vue'
import MainLeftMenu from '@/components/Main/MainLeftMenu.vue'
import MainRightMenu from '@/components/Main/MainRightMenu.vue'
import MainFeed from '@/components/Main/MainFeed.vue'
</script>

<template>
    <MainAppShell>
        <template #header>
            <MainHeader />
        </template>
        <template #left-menu>
            <MainLeftMenu />
        </template>
        <template #right-menu>
            <MainRightMenu />
        </template>
        <template #feed>
            <MainFeed />
        </template>
    </MainAppShell>
</template>

<script lang="ts">
export default {
    name: 'MainView'
}
</script>

<style scoped></style>
