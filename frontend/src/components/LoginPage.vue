<script setup lang="ts">
import { defineProps, ref } from 'vue'

const props = defineProps<{
  errorMessage?: string
  onLoginClicked: (username: string, password: string) => void
}>()

const username = ref('')
const password = ref('')
</script>

<template>
  <div class="login-container" role="main">
    <div v-if="props.errorMessage" role="alert" class="error-message">
      {{ props.errorMessage }}
    </div>
    <h2 id="loginFormLabel">Login</h2>
    <div class="input-group">
      <label for="username" id="usernameLabel">Username:</label>
      <input
        v-model="username"
        type="text"
        id="username"
        name="username"
        aria-labelledby="usernameLabel"
        aria-required="true"
        required
      />
    </div>
    <div class="input-group">
      <label for="password" id="passwordLabel">Password:</label>
      <input
        v-model="password"
        type="password"
        id="password"
        name="password"
        aria-labelledby="passwordLabel"
        aria-required="true"
        required
      />
    </div>
    <button @click="props.onLoginClicked(username, password)" type="submit" class="login-button">
      Login
    </button>
  </div>
</template>

<style scoped>
body {
  font-family: Arial, sans-serif;
  margin: 0;
  padding: 0;
  background-color: #e6f7ff;
  /* Very light blue */
}

.error-message {
  color: red;
  margin-bottom: 15px;
}

.login-container {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100vh;
  flex-direction: column;
}

.login-form {
  background-color: white;
  padding: 20px;
  border-radius: 5px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  width: 300px;
}

.input-group {
  margin-bottom: 15px;
}

input {
  width: calc(100% - 22px);
  padding: 10px;
  margin-top: 5px;
  border: 1px solid #ccc;
  border-radius: 5px;
}

.login-button {
  width: 100%;
  padding: 10px;
  border: none;
  border-radius: 5px;
  background-color: #007bff;
  color: white;
  cursor: pointer;
}

.login-button:hover {
  background-color: #0056b3;
}
</style>
