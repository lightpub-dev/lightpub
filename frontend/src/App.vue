<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import HelloWorld from './components/HelloWorld.vue'
import PostCreatePopup from '@/components/post/CreatePost.vue'
import { ref } from 'vue'

const createShown = ref(true)

const onCreateClick = () => {
  createShown.value = !createShown.value
}
</script>

<template>
  <v-app>
    <header>
      <div class="top-bar">
        <span class="top-bar-text">Lightpub</span>
        <button class="create-icon" @click="onCreateClick">
          <v-icon icon="mdi-pencil" />
        </button>
      </div>
    </header>

    <PostCreatePopup :shown="createShown" />

    <RouterView />
  </v-app>
</template>

<style scoped>
.top-bar {
  background-color: #007bff;
  color: white;
  text-align: center;
  padding: 10px 0;
  font-size: 24px;
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  grid-template-rows: auto;
}

.top-bar-text {
  grid-column: 2;
}

.create-icon {
  height: min-content;
  width: min-content;
  grid-column: 3;
  text-align: right;
  justify-self: end;
  margin-right: 10px;
}

.posts-container {
  margin: 20px;
}
</style>
