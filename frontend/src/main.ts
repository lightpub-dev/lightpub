import App from '@/App.vue'
import '@/style.css'
import { library } from '@fortawesome/fontawesome-svg-core'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import { createApp } from 'vue'
import InfiniteLoading from 'v3-infinite-loading'
import 'v3-infinite-loading/lib/style.css' //required if you're not going to override default slots

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
    },
    {
        path: '/user/:id/followers',
        component: MainView,
        props: {
            mode: 'followers'
        }
    },
    {
        path: '/user/:id/followings',
        component: MainView,
        props: {
            mode: 'followings'
        }
    },
    {
        path: '/post/:id',
        component: MainView,
        props: {
            mode: 'post-detail'
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
    .component('infinite-loading', InfiniteLoading)
    .use(router)
    .mount('#app')
