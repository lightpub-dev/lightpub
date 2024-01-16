<script lang="ts" setup>
import MainLeftMenu from './MainLeftMenu.vue';

</script>
<template>
    <Transition name="from-left">
        <div v-if="isOpen" class="drawer-menu-wrapper-left">
            <div class="main-left-menu-modal-mobile
                fixed inset-0
                z-20
                flex flex-col
                overflow-hidden
                w-9/12 h-full
            ">
                <div class="main-left-menu-modal-mobile-content
                    bg-light-base dark:bg-dark-base
                    text-light-text dark:text-dark-text
                    flex flex-col w-full h-full
                    overflow-auto
                ">
                    <MainLeftMenu
                        @create-post="onCreatePostClicked"
                        @on-close="onClose"
                    />
                </div>
            </div>
        </div>
    </Transition>
    <Transition name="fade">
        <div class="main-left-menu-modal-mobile-backdrop
            bg-black bg-opacity-50
            fixed inset-0
            z-10
            w-full h-full"
            @click="onClose"
            v-if="isOpen"
        ></div>
    </Transition>

</template>

<script lang="ts">
export default {
    props: {
        isOpen: {
            type: Boolean,
            required: true,
        }
    },

    data() {
        return {
        }
    },

    methods: {
        onClose() {
            this.$emit("on-close");
        },

        onCreatePostClicked() {
            this.$emit("on-close");
            this.$emit("create-post");
        },
    },

    emits: ['on-close', 'create-post'],

    watch: {

    },
}
</script>

<style lang="css">

.from-left-enter-active, .from-left-leave-active {
    transition: 225ms ease-in-out;
}

.from-left-enter-from {
    transform: translateX(-100vw) translateX(0px);
}

.from-left-leave-to {
    transform: translateX(-100vw) translateX(0px);
}

.drawer-menu-wrapper-left {
    position: absolute;
    z-index: 20;
    top: 0;
    left: 0; /*左に出す場合*/
    width: 50%;
    height: 100%;
    background-color: white;
}

.fade-enter-active, .fade-leave-active {
    transition: opacity 225ms ease-in-out;
}

.fade-enter-from {
    opacity: 0;
}

.fade-leave-to {
    opacity: 0;
}

</style>
