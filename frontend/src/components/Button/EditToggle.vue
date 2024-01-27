<script setup lang="ts">
const props = defineProps<{
    isEditing: boolean
    onEditDone?: () => void
    onEditStart?: () => void
}>()

const onClick = () => {
    if (props.isEditing) {
        if (props.onEditDone) {
            props.onEditDone()
        }
    } else {
        if (props.onEditStart) {
            props.onEditStart()
        }
    }
}
</script>

<template>
    <div @click="onClick" class="inline">
        <svg
            v-if="!props.isEditing"
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            fill="currentColor"
            class="bi bi-pencil-square inline"
            viewBox="0 0 16 16"
        >
            <path
                d="M15.502 1.94a.5.5 0 0 1 0 .706L14.459 3.69l-2-2L13.502.646a.5.5 0 0 1 .707 0l1.293 1.293zm-1.75 2.456-2-2L4.939 9.21a.5.5 0 0 0-.121.196l-.805 2.414a.25.25 0 0 0 .316.316l2.414-.805a.5.5 0 0 0 .196-.12l6.813-6.814z"
            />
            <path
                fill-rule="evenodd"
                d="M1 13.5A1.5 1.5 0 0 0 2.5 15h11a1.5 1.5 0 0 0 1.5-1.5v-6a.5.5 0 0 0-1 0v6a.5.5 0 0 1-.5.5h-11a.5.5 0 0 1-.5-.5v-11a.5.5 0 0 1 .5-.5H9a.5.5 0 0 0 0-1H2.5A1.5 1.5 0 0 0 1 2.5v11z"
            />
        </svg>
        <svg
            v-else
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            fill="currentColor"
            class="bi bi-check inline"
            viewBox="0 0 16 16"
        >
            <path
                d="M10.97 4.97a.75.75 0 0 1 1.07 1.05l-3.99 4.99a.75.75 0 0 1-1.08.02L4.324 8.384a.75.75 0 1 1 1.06-1.06l2.094 2.093 3.473-4.425a.267.267 0 0 1 .02-.022z"
            />
        </svg>
    </div>
</template>

<script lang="ts">
export default {}
</script>
