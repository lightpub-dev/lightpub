<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import HelloWorld from './components/HelloWorld.vue'
</script>

<template>
  <header>
    <div class="top-bar">Lightpub</div>
  </header>

  <RouterView />
</template>

<style scoped>
.top-bar {
  background-color: #007bff;
  color: white;
  text-align: center;
  padding: 10px 0;
  font-size: 24px;
}

.posts-container {
  margin: 20px;
}
</style>
