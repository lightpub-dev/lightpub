import App from '@/App.vue'
import '@/style.css'
import { library } from '@fortawesome/fontawesome-svg-core'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import { createApp } from 'vue'

/* add some free styles */
import LoginView from '@/components/Login/LoginView.vue'
import MainView from '@/components/Main/MainView.vue'

import {
    faBell,
    faGear,
    faHeart,
    faHouse,
    faImage,
    faInbox,
    faMagnifyingGlass,
    faPlus
} from '@fortawesome/free-solid-svg-icons'
import { createRouter, createWebHashHistory } from 'vue-router'

const routes = [
    { path: '/login', component: LoginView },
    {
        path: '/',
        component: MainView,
        props: {
            mode: 'feed'
        }
    },
    {
        path: '/user/:id',
        component: MainView,
        props: {
            mode: 'profile'
        }
    },
    {
        path: '/trend/posts',
        component: MainView,
        props: {
            mode: 'trend-search'
        }
    }
]

const router = createRouter({
    history: createWebHashHistory(),
    routes
})

library.add(
    faHouse,
    faBell,
    faHeart,
    faInbox,
    faGear,
    faMagnifyingGlass,
    faPlus,
    faImage
)
createApp(App)
    .component('font-awesome-icon', FontAwesomeIcon)
    .use(router)
    .mount('#app')
