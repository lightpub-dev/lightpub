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
        // Hashtag: #hashtag, ends with half/full-width space or newline
        .replace(/#([^\s　\n]+)/g, '<a href="/#/trend/posts?hashtag=$1"><span class="post-link">#$1</span></a>')
        // Mention: @username or @username@domain.ne.jp
        .replace(/@(\w+(@[\w\.]+)?)/g, '<a class="post-link" href="/#/user/@$1"><span class="post-link">@$1</span></a>')
        // Link: http://example.com, https://example.com, http://example.com/search?query=1
        // 問題点: &がエスケープされるため&を含むURLがリンクにならない。また、URLの#以降がハッシュタグとして認識される。
        .replace(/(https?:\/\/[\w\.\?\/\=#]+)/g, '<a class="post-link" href="$1" target="_blank">$1</a>')
        .replace(/\n/g, '<br>')
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