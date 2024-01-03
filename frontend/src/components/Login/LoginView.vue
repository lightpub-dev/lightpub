<template>
  <div class="min-h-screen flex items-center justify-center bg-blue-100">
    <div class="bg-white p-6 rounded shadow-md w-80">
      <h2 class="mb-4 text-xl font-bold text-gray-700">Login</h2>
      <p class="text-red-500 text-xs italic mb-2">{{ errorMsg }}</p>
      <input v-model="username" type="text" placeholder="Username" class="mb-3 w-full px-4 py-2 border rounded-lg text-gray-700 focus:outline-none focus:border-blue-500" />
      <input v-model="password" type="password" placeholder="Password" class="mb-3 w-full px-4 py-2 border rounded-lg text-gray-700 focus:outline-none focus:border-blue-500" />
      <button @click="login" class="w-full px-3 py-2 rounded text-white bg-blue-500 focus:bg-blue-600 focus:outline-none">Login</button>
    </div>
  </div>
</template>

<script lang="ts">
import axios from 'axios';
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { storeLoginToken } from '../../auth';
import { BASE_URL } from '../../settings';

export default {
  name: "LoginView",
  setup() {
    const username = ref('');
    const password = ref('');
    const errorMsg = ref('');
    const router = useRouter(); // Define the router object

    const login = async () => {
      try {
        const specialAxios = axios.create({
          baseURL: BASE_URL,
          headers: {
            'Content-Type': 'application/json',
          },
        }); // disable auth header for login
        const response = await specialAxios.post('/login', {
          username: username.value,
          password: password.value,
        });

        // handle response here
        // if status is 200, redirect to home page
        const token = response.data.token;
        // set to localStorage
        storeLoginToken(token);

        // move to "/"
        router.push('/'); // Use the router object to navigate to the home page
      } catch (error) {
        // handle error here
        // if not, show error message
        errorMsg.value = error.response.data;
        return;
      }
    };

    return {
      username,
      password,
      login,
      errorMsg,
    };
  },
};
</script>

<style scoped>
</style>