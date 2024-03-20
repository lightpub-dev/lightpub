import './assets/main.css'

import { createApp } from 'vue'
import { createPinia } from 'pinia'

import App from './App.vue'
import router from './router'
import { createVuetify } from 'vuetify'
import { aliases, md } from 'vuetify/iconsets/md'

const app = createApp(App)

const vuetify = createVuetify({
  icons: {
    defaultSet: 'md',
    aliases,
    sets: {
      md
    }
  }
})

app.use(createPinia())
app.use(router)
app.use(vuetify)

app.mount('#app')
