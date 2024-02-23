<template>
    <div class="container mx-auto">
        <ul v-if="users && users.length" class="space-y-4">
            <li
                v-for="user in users"
                :key="user.id"
                class="flex flex-row items-center bg-white p-4 shadow rounded-lg"
            >
                <img
                    :src="getActualAvatar(user.avatar)"
                    alt="User's Profile Picture"
                    class="w-16 h-16 rounded-full mb-2 cursor-pointer"
                    @click="jumpToProfile(user.id)"
                />
                <h2
                    class="text-lg font-semibold cursor-pointer"
                    @click="jumpToProfile(user.id)"
                >
                    {{ user.nickname }}
                </h2>
                <p
                    class="text-gray-600 mx-4 cursor-pointer"
                    @click="jumpToProfile(user.id)"
                >
                    @{{ user.username }}
                </p>
                <p class="text-gray-500 text-sm">{{ user.bio }}</p>
            </li>
        </ul>
        <p v-else class="text-center text-gray-500">No users found.</p>
    </div>
</template>

<script lang="ts" setup>
import { inject, ref, watchEffect } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { AUTH_AXIOS } from '../../consts'
import { DUMMY_AVATAR_URL } from '../../settings'

const getActualAvatar = (avatar: string | null) => {
    if (avatar) {
        return avatar
    }
    return DUMMY_AVATAR_URL
}

interface User {
    id: string
    avatar: string
    nickname: string
    username: string
    bio: string
}

const props = defineProps<{
    mode: 'followers' | 'followings'
}>()

const axios = inject(AUTH_AXIOS)!
const route = useRoute()
const router = useRouter()

const targetUserId = route.params.id as string

const users = ref<User[]>([])

const fetchUsers = async () => {
    let url = ''
    switch (props.mode) {
        case 'followers':
            url = `/followers?user=${targetUserId}`
            break
        case 'followings':
            url = `/followings?user=${targetUserId}`
            break
    }
    const res = await axios.get(url)
    switch (props.mode) {
        case 'followers':
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            users.value = res.data.results.map((r: any) => r.follower)
            break
        case 'followings':
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            users.value = res.data.results.map((r: any) => r.followee)
            break
    }
}

watchEffect(() => {
    fetchUsers()
})

const jumpToProfile = (userId: string) => {
    router.push(`/user/${userId}`)
}
</script>
