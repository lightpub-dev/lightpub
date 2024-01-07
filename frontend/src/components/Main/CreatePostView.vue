<script setup lang="ts">
import { ref, watch } from 'vue'

const props = defineProps({
    showPostMenu: {
        type: Boolean,
        required: true
    }
})

const showPostMenu = ref(props.showPostMenu)
const tweetText = ref('')
const selectedImage = ref<File | null>(null)

watch(
    () => props.showPostMenu,
    value => {
        showPostMenu.value = value
    }
)

const selectImage = (event: Event) => {
    const files = (event.target as HTMLInputElement).files
    if (files) {
        selectedImage.value = files[0]
    }
}

const postTweet = () => {
    alert('Tweet Posted!')
    alert("NO, IT'S NOT ACTUALLY POSTED!")
    tweetText.value = ''
    selectedImage.value = null
    closePostMenu()
}

const closePostMenu = () => {
    showPostMenu.value = false
}
</script>

<template>
    <transition name="transit">
        <div
            v-if="showPostMenu"
            class="w-full p-6 mx-auto border border-gray-300 rounded-xl shadow-lg bg-white"
        >
            <textarea
                v-model="tweetText"
                placeholder="What's happening?"
                class="w-full h-32 p-4 mb-4 text-lg border-0 rounded-lg resize-none bg-gray-100 dark:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-300"
            ></textarea>
            <div class="flex items-center justify-between">
                <div class="flex items-center space-x-2">
                    <input
                        type="file"
                        @change="selectImage"
                        class="hidden"
                        ref="fileInput"
                        accept="image/*"
                    />
                    <button
                        @click="$refs.fileInput.click()"
                        class="flex items-center justify-center px-4 py-2 space-x-2 border border-blue-500 text-blue-500 rounded-lg transition duration-300 ease-in-out hover:bg-blue-500 hover:text-white"
                    >
                        <font-awesome-icon :icon="['fa-solid', 'fa-image']" />
                        <span>Add Image</span>
                    </button>
                </div>
                <div class="flex space-x-4">
                    <button
                        @click="closePostMenu"
                        class="px-4 py-2 border border-gray-400 rounded-lg transition duration-300 ease-in-out hover:bg-gray-200"
                    >
                        Cancel
                    </button>
                    <button
                        @click="postTweet"
                        class="px-4 py-2 text-white bg-green-500 rounded-lg transition duration-300 ease-in-out hover:bg-green-600"
                    >
                        Tweet
                    </button>
                </div>
            </div>
        </div>
    </transition>
</template>

<style scoped>
.transit-enter-active,
.transit-leave-active {
    transition: opacity 0.2s ease-in-out;
}
.transit-enter-from,
.transit-leave-to {
    opacity: 0;
}
</style>
