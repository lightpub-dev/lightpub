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

.post {
  background-color: white;
  border-radius: 5px;
  margin-bottom: 20px;
  padding: 15px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.post-header {
  margin-bottom: 10px;
}

.nickname,
.username,
.hostname,
.created-date {
  display: block;
  margin-bottom: 5px;
}

.nickname {
  font-weight: bold;
}

.username {
  color: #555;
}

.hostname {
  color: #777;
  font-size: smaller;
}

.created-date {
  color: #999;
  font-size: smaller;
}

.post-text {
  font-size: 16px;
}
</style>
