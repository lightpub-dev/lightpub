<template>
    <div class="min-h-screen flex items-center justify-center bg-blue-100">
        <div class="bg-white p-6 rounded shadow-md w-80">
            <h2 class="mb-4 text-xl font-bold text-gray-700">Register</h2>
            <p class="text-red-500 text-xs italic mb-2">{{ errorMsg }}</p>
            <input
                v-model="username"
                type="text"
                placeholder="Username"
                class="mb-3 w-full px-4 py-2 border rounded-lg text-gray-700 focus:outline-none focus:border-blue-500"
            />
            <input
                v-model="nickname"
                type="text"
                placeholder="Nickname"
                class="mb-3 w-full px-4 py-2 border rounded-lg text-gray-700 focus:outline-none focus:border-blue-500"
            />
            <input
                v-model="password"
                type="password"
                placeholder="Password"
                class="mb-3 w-full px-4 py-2 border rounded-lg text-gray-700 focus:outline-none focus:border-blue-500"
            />
            <button
                @click="register"
                class="w-full px-3 py-2 rounded text-white bg-blue-500 focus:bg-blue-600 focus:outline-none"
            >
                Register
            </button>
        </div>
    </div>
</template>

<script lang="ts">
import axios from 'axios'
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { storeLoginToken, storeUsername } from '../../auth'
import { BASE_URL } from '../../settings'

export default {
    name: 'RegisterView',
    setup() {
        const username = ref('')
        const nickname = ref('')
        const password = ref('')
        const errorMsg = ref('')
        const router = useRouter() // Define the router object

        const register = async () => {
            try {

                const specialAxios = axios.create({
                    baseURL: BASE_URL,
                    headers: {
                        'Content-Type': 'application/json'
                    }
                }) // disable auth header for login

                const response = await specialAxios.post('/register', {
                    username: username.value,
                    password: password.value,
                    nickname: nickname.value
                })

                // handle response here

                // move to "/registration-success" page
                router.push('/registration-success') // Use the router object to navigate to the home page
                // eslint-disable-next-line @typescript-eslint/no-explicit-any
            } catch (error: any) {
                // handle error here
                // if not, show error message
                errorMsg.value = error.response.data
                return
            }
        }

        return {
            username,
            password,
            nickname,
            register,
            errorMsg
        }
    }
}
</script>

<style scoped></style>
