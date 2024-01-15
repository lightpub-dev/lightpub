<template>
    <div class="container mx-auto">
        <ul v-if="users && users.length" class="space-y-4">
            <li
                v-for="user in users"
                :key="user.id"
                class="flex flex-col items-center bg-white p-4 shadow rounded-lg"
            >
                <img
                    :src="user.picture"
                    alt="User's Profile Picture"
                    class="w-16 h-16 rounded-full mb-2"
                />
                <h2 class="text-lg font-semibold">{{ user.displayName }}</h2>
                <p class="text-gray-600">@{{ user.username }}</p>
                <p class="text-gray-500 text-sm">{{ user.bio }}</p>
            </li>
        </ul>
        <p v-else class="text-center text-gray-500">No users found.</p>
    </div>
</template>

<script lang="ts" setup>
import { inject, onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { AUTH_AXIOS } from '../../consts'

interface User {
    id: number
    picture: string
    displayName: string
    username: string
    bio: string
}

const props = defineProps<{
    mode: 'followers' | 'following'
}>()

const axios = inject(AUTH_AXIOS)!
const route = useRoute()

const targetUserId = route.params.id as string

const users = ref<User[]>([])

const fetchUsers = async () => {
    // Replace with your fetch routine
    const res = await axios.get(`/users/${targetUserId}/${props.mode}`)
}

onMounted(fetchUsers)

return { users }
</script>
