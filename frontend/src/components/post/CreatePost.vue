<script setup lang="ts">
import { emitter } from '@/event'
import { axiosOptions } from '@/store'
import axios from 'axios'
import { ref } from 'vue'

const postContent = ref('')
const selectedPrivacy = ref('public')

const props = defineProps<{
  shown: boolean
}>()

async function sendPost() {
  // Logic to send the post (you'd likely use an HTTP library like Axios)
  await axios.post(
    '/post',
    {
      content: postContent.value,
      privacy: selectedPrivacy.value
    },
    axiosOptions()
  )

  emitter.emit('newPostCreated')

  // Clear the input after sending
  postContent.value = ''
}
</script>

<template>
  <div v-if="shown" class="post-create-page">
    <textarea v-model="postContent" placeholder="What's on your mind?"></textarea>

    <select v-model="selectedPrivacy">
      <option value="public">Public</option>
      <option value="unlisted">Unlisted</option>
      <option value="follower">Follower-only</option>
      <option value="private">Private</option>
    </select>

    <button @click="sendPost">Send</button>
  </div>
</template>

<style scoped>
.post-create-popup {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background-color: white;
}
</style>
