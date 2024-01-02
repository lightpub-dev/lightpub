import {createApp} from "vue";
import "@/style.css";
import App from "@/App.vue";
import {FontAwesomeIcon} from "@fortawesome/vue-fontawesome";
import {library} from '@fortawesome/fontawesome-svg-core'

/* add some free styles */
import {faBell, faGear, faHeart, faHouse, faInbox, faMagnifyingGlass} from "@fortawesome/free-solid-svg-icons";

library.add(faHouse, faBell, faHeart, faInbox, faGear, faMagnifyingGlass);
createApp(App)
    .component("font-awesome-icon", FontAwesomeIcon)
    .mount("#app");
