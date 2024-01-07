<script lang="ts" setup>
import { computed, inject, ref } from 'vue'
import { useRoute } from 'vue-router'
import { AUTH_AXIOS } from '../../consts'

const route = useRoute()
const id = route.params.id
const axios = inject(AUTH_AXIOS)!

const username = ref('')
const nickname = ref('')
const hostname = ref('')
const bio = ref('')
const labels = ref<
    {
        key: string
        value: string
    }[]
>([])

const nPosts = ref(100)
const nFollowers = ref(50)
const nFollowings = ref(51)

const fetchProfile = async () => {
    const res = await axios.get(`/user/${id}`)
    username.value = res.data.username
    nickname.value = res.data.nickname
    hostname.value = res.data.hostname
    bio.value = res.data.bio
    labels.value = res.data.labels
}

fetchProfile()

const atHostname = computed(() => {
    if (hostname.value) {
        return `@${hostname.value}`
    } else {
        return ''
    }
})
</script>

<template>
    <div
        class="min-h-screen bg-[rgb(219,234,254)] flex items-start justify-center pt-10"
    >
        <div class="bg-white p-6 rounded-lg shadow-lg w-64">
            <div class="mb-4">
                <img
                    class="w-20 h-20 mx-auto rounded-full"
                    src="https://placekitten.com/200/200"
                    alt="User icon"
                />
            </div>
            <div class="text-center">
                <h2 class="text-xl font-semibold text-gray-700 mb-2">
                    {{ nickname }}
                </h2>
                <h3 class="text-gray-500 mb-3">
                    @{{ username }}{{ atHostname }}
                </h3>
                <p class="text-gray-600">{{ bio }}</p>
            </div>
            <!-- Specification Table -->
            <div class="mt-4">
                <table class="w-full text-sm text-center text-gray-500">
                    <tbody>
                        <tr
                            class="bg-white border-b"
                            v-for="label in labels"
                            :key="label.key"
                        >
                            <th scope="row" class="px-2 py-1">
                                {{ label.key }}
                            </th>
                            <td class="px-2 py-1">{{ label.value }}</td>
                        </tr>
                    </tbody>
                </table>
            </div>
            <!-- Counter Section -->
            <div class="flex justify-between mt-4">
                <div class="text-center">
                    <p class="text-sm text-gray-600">Posts</p>
                    <p class="font-semibold text-gray-700">{{ nPosts }}</p>
                </div>
                <div class="text-center">
                    <p class="text-sm text-gray-600">Followers</p>
                    <p class="font-semibold text-gray-700">{{ nFollowers }}</p>
                </div>
                <div class="text-center">
                    <p class="text-sm text-gray-600">Following</p>
                    <p class="font-semibold text-gray-700">{{ nFollowings }}</p>
                </div>
            </div>
        </div>
    </div>
</template>
