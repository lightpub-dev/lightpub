import App from "@/App.vue";
import "@/style.css";
import { library } from '@fortawesome/fontawesome-svg-core';
import { FontAwesomeIcon } from "@fortawesome/vue-fontawesome";
import { createApp } from "vue";

/* add some free styles */
import LoginView from "@/components/Login/LoginView.vue";
import MainView from "@/components/Main/MainView.vue";

import { faBell, faGear, faHeart, faHouse, faInbox, faMagnifyingGlass } from "@fortawesome/free-solid-svg-icons";
import axios from "axios";
import { createRouter, createWebHashHistory } from "vue-router";
import { getLoginToken } from "./auth";
import { BASE_URL } from "./settings";

// axios setup
axios.defaults.baseURL = BASE_URL;
axios.interceptors.request.use((config) => {
    const token = getLoginToken();
    if (token) {
        config.headers.Authorization = `Bearer ${token}`;
    }

    return config;
});

const routes = [
    {path: "/login", component: LoginView},
    {path: "/", component: MainView}
]

const router = createRouter({
    history: createWebHashHistory(),
    routes
})

library.add(faHouse, faBell, faHeart, faInbox, faGear, faMagnifyingGlass);
createApp(App)
    .component("font-awesome-icon", FontAwesomeIcon)
    .use(router)
    .mount("#app");
