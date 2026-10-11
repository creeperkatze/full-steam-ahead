import '@fontsource-variable/inter'
import '../../assets/main.css'

import { createApp } from 'vue'

import { commands } from '../../bindings'
import { i18n } from '../../i18n'
import App from './App.vue'
import { router } from './router'

document.addEventListener('contextmenu', (event) => event.preventDefault())

createApp(App).use(router).use(i18n).mount('#app')

void commands.showMainWindow()
