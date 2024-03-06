<script setup lang="ts">

/**
 * This component is used to display the content of a user post.
 */

import { computed, defineProps } from 'vue'

// This function escapes HTML characters in a string.
const escapeHTML = (unsafe: string) => {
    return unsafe
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#039;')
}

// Define the props for this component.
const props = defineProps<{
    content: string
}>()

// This computed property processes the content of the post.
const processedContent = computed(() => {
    return escapeHTML(props.content)
        .replace(/\n/g, '<br>')
        .replace(/#([^\s　]+)/g, '<a href="/#/trend/posts?hashtag=$1"><span class="post-link">#$1</span></a>')
        .replace(/@(\w+)/g, '<a class="post-link" href="/#/user/@$1"><span class="post-link">@$1</span></a>')
        .replace(/(https?:\/\/[^\s]+)/g, '<a class="post-link" href="$1" target="_blank">$1</a>')
})

</script>

<template>
    <p class="pt-5 text-gray-600 text-lg mb-4" v-html="processedContent" />
</template>

<style>
.post-link {
    color: #3385d2;
    text-decoration: none;
    font-weight: bold;
}
</style>