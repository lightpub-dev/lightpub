<template>
    <div class="flex w-full h-full justify-between items-center">
        <div class="flex items-center">
            <MenuButtonMobile @click="onMenuButtonClicked" />
            <TrendButtonMobile @click="onTrendButtonClicked" />
        </div>

        <div class="flex items-center">
            <p class="text-xl font-normal font-bold">LightPub</p>
        </div>
    </div>
</template>
<script lang="ts" setup>
import MenuButtonMobile from './MenuButtonMobile.vue';
import TrendButtonMobile from './TrendButtonMobile.vue';
import { defineEmits } from 'vue';

const emit = defineEmits(["on-menu-button-clicked", "on-trend-button-clicked"]);

const onMenuButtonClicked = () => {
    emit("on-menu-button-clicked");
};

const onTrendButtonClicked = () => {
    emit("on-trend-button-clicked");
};

</script>
