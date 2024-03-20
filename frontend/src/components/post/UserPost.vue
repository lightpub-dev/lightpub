<template>
  <div class="post">
    <div class="post-header">
      <div class="author-line">
        <span class="nickname">{{ props.nickname }}</span>
        <span class="username"
          >@{{ props.username }}<span v-if="props.host">@{{ props.host }}</span></span
        >
        <span class="created-at">{{ props.createdAt.toLocaleString() }}</span>
      </div>
    </div>
    <div class="post-content">{{ props.content }}</div>
    <div>
      <button>Reply</button>
      <button>Repost</button>
      <button>Like</button>
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  nickname: string
  username: string
  host?: string
  createdAt: Date
  content?: string
}>()

export interface UserPost {
  id: string
  author: {
    nickname: string
    username: string
    host?: string
  }
  content: string
  createdAt: Date
}
</script>

<style scoped>
.post {
  background-color: white;
  border-radius: 5px;
  margin-bottom: 20px;
  padding: 15px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.author-line {
  display: flex;
  flex-direction: row;
}

.post-header {
  margin-bottom: 10px;
}

.nickname,
.username,
.host,
.created-at {
  display: block;
  margin-bottom: 5px;
}

.nickname {
  font-weight: bold;
}

.username {
  color: #555;
  margin-left: 1em;
}

.host {
  color: #777;
  font-size: smaller;
}

.created-at {
  color: #999;
  font-size: smaller;
}

.post-content {
  font-size: 16px;
}
</style>
