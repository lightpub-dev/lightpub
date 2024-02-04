<script lang="ts" setup>
import axios from 'axios'
import { provide, ref } from 'vue'
import { getLoginToken, getUsername } from '../../auth'
import { AUTH_AXIOS, CURRENT_USERNAME, DEVICE_TYPE } from '../../consts'
import { BASE_URL } from '../../settings'
import { eventBus } from '../../event'

import MainViewMobile from '@/components/Main/MainViewMobile.vue'
import MainViewDesktop from '@/components/Main/MainViewDesktop.vue'

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

const deviceType = ref('mobile')

const checkDevice = () => {
    if (window.innerWidth < 768) {
        deviceType.value = 'mobile'
    } else {
        deviceType.value = 'desktop'
    }
}

window.addEventListener('resize', checkDevice)

checkDevice()

// provides deviceType to children
provide(AUTH_AXIOS, authAxios)
provide(CURRENT_USERNAME, username)
provide(DEVICE_TYPE, deviceType)

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
    <MainViewMobile v-if="deviceType === 'mobile'" :mode="props.mode" />
    <MainViewDesktop v-else :mode="props.mode" />
</template>

<script lang="ts">
export default {
    name: 'MainView'
}
</script>

<style scoped></style>
