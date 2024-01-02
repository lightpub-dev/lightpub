<script lang="ts" setup>


import {reactive} from 'vue'

const trendings = reactive([
    {
        index: 0,
        gender: 'Software',
        label: '#GitHub',
        stats: '10 Posts'
    },
    {
        index: 0,
        gender: 'Entertainment',
        label: '#OpenAirInterface',
        stats: '8 Posts'
    },
    {
        index: 0,
        gender: 'Music',
        label: '#IjimeDameZettai',
        stats: '2 Posts'
    },
    {
        index: 0,
        gender: 'Technology',
        label: '#React',
        stats: '1 Posts'
    },

])


</script>
<template>
    <div class="w-full h-full flex flex-col py-2 px-2 bg-gray-100 overflow-y-auto">

        <ul class="w-full bg-green-100 p-5 rounded-lg shadow-xl mb-5">
            <li class="text-green-600 text-lg font-semibold">
                <p>Trends</p>

            </li>
            <div class="my-2 border-b border-green-600 w-full"></div>
            <li v-for="(trending, index) in trendings" :key="index"
                class="px-2 py-2 last:mb-0 hover:bg-green-300 rounded-md transition-colors duration-200">
                <p class="text-xs text-gray-500 dark:text-gray-400">{{ index + 1 }} - {{ trending.gender }} </p>
                <p class="text-lg dark:text-gray-100 text-gray-800 font-bold">{{ trending.label }}</p>
                <p class="text-xs text-gray-500 dark:text-gray-400">{{ trending.stats }}</p>
            </li>
        </ul>

    </div>
</template>


<script lang="ts">
export default {}
</script>
