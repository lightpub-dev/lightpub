<script setup lang="ts">
import { inject, ref, watch } from 'vue'
import { AUTH_AXIOS } from '../../consts'
import { eventBus } from '../../event'

const emit = defineEmits<{
    (e: 'created'): void
    (e: 'canceled'): void
    (e: 'on-close'): void
}>()

const props = defineProps<{
    showPostMenu: boolean
    replyToId: string | null
}>()

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

const authedAxios = inject(AUTH_AXIOS)!

const postTweet = async () => {
    // upload image if selected
    let uploadId = null
    if (selectedImage.value) {
        const formData = new FormData()
        formData.append('file', selectedImage.value)
        try {
            const res = await authedAxios.post('/uploads', formData, {
                headers: {
                    'Content-Type': 'multipart/form-data'
                }
            })
            uploadId = res.data.id
        } catch (ex) {
            console.error(ex)
            alert('Failed to upload image')
            return
        }
    }

    const content = tweetText.value
    const privacy = 0 // TODO

    try {
        const req = {
            content,
            privacy,
            reply_to_id: props.replyToId
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
        } as any
        if (uploadId) {
            req['attached_uploads'] = [uploadId]
        }
        await authedAxios.post('/posts', req)

        tweetText.value = ''
        selectedImage.value = null
        closePostMenu()

        emit('created')
        eventBus.emit('post-created')
    } catch (ex) {
        console.error(ex)
        alert('Failed to post tweet')
    }
}

const closePostMenu = () => {
    emit('canceled')
    showPostMenu.value = false
    emit('on-close')
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
                class="w-full h-32 p-4 mb-4 text-lg border-0 rounded-lg resize-none bg-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-300"
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
                        @click="($refs.fileInput as any).click()"
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
