<template>
  <LoginPage :onLoginClicked="onLoginClicked" :errorMessage="errorMessage" />
</template>

<script setup lang="ts">
import { defineComponent, ref } from 'vue'
import LoginPage from '../components/LoginPage.vue'
import axios from 'axios'
import router from '@/router'
import { useAuthStore } from '@/store'

// This example assumes you have a LoginPage component
// that takes a prop 'onLoginClicked' which is a function.
defineComponent({
  components: {
    LoginPage
  }
})

const errorMessage = ref<string | undefined>(undefined)
const authStore = useAuthStore()

const onLoginClicked = async (username: string, password: string) => {
  try {
    const response = await axios.post('/login', {
      username: username,
      password: password
    })

    if (response.data.token) {
      // Login successful
      errorMessage.value = undefined
      const token = response.data.token
      // Store token in pinia
      authStore.token = token

      router.push('/timeline') // Redirect to timeline
    } else {
      // Handle unsuccessful login (e.g., display an error message)
      console.error('Login failed - no token in response')
      errorMessage.value = 'Login failed. Username or password is incorrect.'
      authStore.token = null
    }
  } catch (error) {
    // Handle API request error (e.g., network issues, server errors)
    console.error('Login API Error:', error)
    authStore.token = null
  }
}
</script>

<style scoped>
/* Your scoped CSS styles for LoginView.vue */
</style>
