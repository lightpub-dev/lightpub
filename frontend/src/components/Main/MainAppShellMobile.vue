<script lang="ts" setup>

</script>

<template>
    <div class="light-mobile light">
        <div
            class="w-screen h-screen bg-light-base dark:bg-dark-base flex flex-col text-gray-500"
        >
            <div class="flex w-full h-full relative overflow-hidden">
                <div class="flex flex-col w-full h-full">
                    <div
                        class="flex items-center justify-between w-full h-14 border-b border-light-border dark:border-dark-border px-5"
                    >
                        <slot name="header"></slot>
                    </div>
                    <div class="flex flex-col w-full h-full overflow-auto">
                        <slot name="create-post"></slot>
                        <slot name="feed"></slot>
                    </div>
                </div>
                <slot name="modals"></slot>
            </div>
        </div>
    </div>
</template>

<script lang="ts">
export default {

}
</script>
<style lang=""></style>
