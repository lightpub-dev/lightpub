<template>
    <div class="min-h-screen flex items-center justify-center bg-blue-100">
        <div class="bg-white p-6 rounded shadow-md w-80">
            <div>
                <h2 class="mb-4 text-xl font-bold text-gray-700">
                    You have successfully registered!
                </h2>
                <p>
                    <router-link to="/login" class="text-blue-500">Login</router-link>
                </p>
            </div>
        </div>
    </div>
</template>

<script lang="ts">

export default {
    name: 'RegistrationSuccessView',
    
    setup() {
        return {}
    }
}
</script>

<style scoped></style>
