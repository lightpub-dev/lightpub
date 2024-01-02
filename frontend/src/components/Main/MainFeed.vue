<script lang="ts" setup>
import {reactive} from 'vue';
import {Posts} from '../Post/processFeedPosts.ts';
const feedPosts = reactive(Posts);
</script>

<template>
    <div class="grid-cols-1 w-full grid md:grid-cols-1 px-20 pt-5 transition-all bg-gray-100">
        <div class="flex flex-col p-2 ">
            <Post v-for="(post, index) in feedPosts" :key="index" :user_post="post"></Post>
        </div>
    </div>
</template>

<script lang="ts">
import Post from "@/components/Post/Post.vue";

export default {
    components: {Post}
}
</script>
