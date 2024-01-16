<script lang="ts" setup>

</script>

<template>
    <div class="light">
        <div
            class="w-screen h-screen bg-light-base dark:bg-dark-base flex flex-col text-gray-500"
        >
            <div class="flex w-full h-full relative overflow-hidden">
                <div
                    class="absolute inset-y-0 left-0 z-10 flex flex-col w-2/12 border-r border-light-border dark:border-dark-border md:relative md:w-110 overflow-x-hidden transition-all"
                >
                    <slot name="left-menu"></slot>
                </div>
                <div class="flex flex-col w-7/12 h-full">
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
                <div
                    class="flex flex-col w-3/12 h-full border-l border-light-border dark:border-dark-border bg-light-neutral dark:bg-dark-neutral transition-all overflow-x-hidden"
                >
                    <slot name="right-menu"></slot>
                </div>
            </div>
        </div>
    </div>
</template>

<script lang="ts">
export default {}
</script>
<style lang=""></style>
