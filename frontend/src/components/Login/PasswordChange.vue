<script lang="ts" setup>
import { inject, ref } from 'vue'
import { AUTH_AXIOS } from '../../consts'
import { getUsername } from '../../auth'
import { useRouter } from 'vue-router'

const axios = inject(AUTH_AXIOS)!
const username = getUsername()

const newPassword = ref('')
const confirmPassword = ref('')

const router = useRouter()

const submitPasswordChange = async () => {
    // Add your password change logic here
    if (newPassword.value !== confirmPassword.value) {
        alert('Passwords do not match')
        return
    }

    try {
        await axios.patch(`/users/@${username}/`, {
            password: newPassword.value
        })
    } catch (ex) {
        console.error(ex)
        alert('Failed to update password')
        return
    }

    alert('Password changed successfully')
    newPassword.value = ''
    confirmPassword.value = ''

    router.push('/login')
}
</script>

<template>
    <form @submit.prevent="submitPasswordChange" class="space-y-4">
        <div>
            <label
                for="newPassword"
                class="block text-sm font-medium text-gray-700"
                >New Password</label
            >
            <input
                id="newPassword"
                v-model="newPassword"
                type="password"
                required
                class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50"
                placeholder="Enter new password"
            />
        </div>

        <div>
            <label
                for="confirmPassword"
                class="block text-sm font-medium text-gray-700"
                >Confirm Password</label
            >
            <input
                id="confirmPassword"
                v-model="confirmPassword"
                type="password"
                required
                class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50"
                placeholder="Confirm new password"
            />
        </div>

        <div>
            <button
                type="submit"
                class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
            >
                Change Password
            </button>
        </div>
    </form>
</template>
