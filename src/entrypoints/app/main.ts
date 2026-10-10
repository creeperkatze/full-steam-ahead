import '@fontsource-variable/inter'
import '../../assets/main.css'

import { createApp } from 'vue'

import { commands } from '../../bindings'
import { roundWindowCorners } from '../../helpers/roundedCorners'
import { i18n } from '../../i18n'
import App from './App.vue'
import { router } from './router'

document.addEventListener('contextmenu', (event) => event.preventDefault())

if (window.__FSA_ROUNDED_CORNERS__) void roundWindowCorners()

createApp(App).use(router).use(i18n).mount('#app')

void commands.showMainWindow()
