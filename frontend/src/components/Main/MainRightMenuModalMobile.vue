<script lang="ts" setup>
import MainRightMenu from './MainRightMenu.vue';

</script>
<template>
    <Transition name="from-right">
        <div v-if="isOpen" class="drawer-menu-wrapper-right">
            <div class="main-right-menu-modal-mobile
                flex flex-col
                overflow-hidden
                w-full h-full
            ">
                <div class="main-right-menu-modal-mobile-content
                    bg-light-base dark:bg-dark-base
                    text-light-text dark:text-dark-text
                    flex flex-col w-full h-full
                    overflow-auto
                ">
                    <MainRightMenu
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
        },
    },

    emits: ['on-close'],

    watch: {

    },
}
</script>

<style lang="css">

.from-right-enter-active, .from-right-leave-active {
    transition: 225ms ease-in-out;
}

.from-right-enter-from {
    transform: translateX(100vw) translateX(0px);
}

.from-right-leave-to {
    transform: translateX(100vw) translateX(0px);
}

.drawer-menu-wrapper-right {
    position: absolute;
    z-index: 20;
    top: 0;
    right: 0;
    width: 75%;
    height: 100%;
    background-color: white;
}

</style>
