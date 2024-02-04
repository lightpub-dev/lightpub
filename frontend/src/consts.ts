import { AxiosInstance } from 'axios'
import { InjectionKey, Ref } from 'vue'

export const AUTH_AXIOS = Symbol('authAxios') as InjectionKey<AxiosInstance>
export const CURRENT_USERNAME = Symbol(
    'currentUsername'
) as InjectionKey<string>

export const DEVICE_TYPE = Symbol('deviceType') as InjectionKey<Ref<string>>