import { defineStore } from 'pinia'

export const useAuthStore = defineStore('auth', {
  state: () => {
    return {
      token: null as string | null
    }
  },
  getters: {
    axiosOptions(state) {
      return {
        headers: {
          authorization: `Bearer ${state.token}`
        }
      }
    }
  }
})
