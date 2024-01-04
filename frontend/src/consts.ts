import { AxiosInstance } from 'axios'
import { InjectionKey } from 'vue'

export const AUTH_AXIOS = Symbol('authAxios') as InjectionKey<AxiosInstance>
