<script setup lang="ts">
import { computed, inject } from 'vue';
import { AUTH_AXIOS } from '../../consts';
import { eventBus } from '../../event';

const props = defineProps<{
    emojiName: string,
    count: number,
    reactedByMe?: boolean,
    postId?: string,
}>();

const emojiIcon = computed(() => String.fromCodePoint(parseInt(props.emojiName, 16)))

const axios = inject(AUTH_AXIOS)!;
const onClick = async () => {
    if (!props.postId) return;

    if (props.reactedByMe) {
        // delete
        try {
            await axios.delete(`/reactions/${props.postId}`, {
                params: {
                    emoji: props.emojiName
                }
            })
            eventBus.emit('reaction-deleted');
        } catch (e) {
            console.error(e);
            alert('Failed to delete reaction')
        }
    } else {
        // create
        try {
            await axios.post(`/reactions`, {
                emoji: props.emojiName,
                post: props.postId,
            })
            eventBus.emit('reaction-created');
        } catch (e) {
            console.error(e);
            alert('Failed to create reaction')
        }
    }
}
</script>

<template>
    <div class="flex items-center space-x-2 reaction-badge cursor-pointer" :class="{ 'bg-orange-100': props.reactedByMe }"
        @click="onClick">
        <span class="text-2xl">{{ emojiIcon }}</span>
        <span class="text-sm">{{ props.count }}</span>
    </div>
</template>

<style>
.reaction-badge {
    /* Light gray background */
    border-radius: 10px;
    /* Rounded corners */
    box-shadow: 5px 5px 10px rgba(0, 0, 0, 0.2);
    /* Outer shadow */
    /* -5px -5px 10px rgba(255, 255, 255, 0.7); */
    /* Inner light */
    padding: 5px;
    /* Spacing around the content */
    width: fit-content;
    height: fit-content;
}
</style>