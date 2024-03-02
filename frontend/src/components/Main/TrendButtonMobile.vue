<template>
    <div class="flex items-center">
        <button class="flex items-center justify-center w-10 h-10 rounded-full bg-light-neutral dark:bg-dark-neutral text-light-text dark:text-dark-text"
            @click="onButtonClicked"
        >
            <svg
                class="w-5 h-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
            >
                <!-- icon is "#"-like shape -->
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M4 8h16M4 16h16 M8 4v16M16 4v16"
                ></path>
            </svg>
        </button>
    </div>
</template>

<script lang="ts" setup>
// emits event when the button is clicked

import { defineEmits } from 'vue';

const emit = defineEmits(["click"]);

const onButtonClicked = () => {
    emit("click");
};

</script>