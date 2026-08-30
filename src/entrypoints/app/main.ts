import '../../assets/main.css'

import { createApp } from 'vue'

import { api } from '../../helpers/api'
import { i18n } from '../../i18n'
import App from './App.vue'
import { router } from './router'

createApp(App).use(router).use(i18n).mount('#app')

void api.showMainWindow()
