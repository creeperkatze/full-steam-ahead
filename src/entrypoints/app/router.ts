import { createRouter, createWebHashHistory } from "vue-router";
import ImportFlowView from "./views/ImportFlowView.vue";
import SettingsView from "./views/SettingsView.vue";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", name: "import", component: ImportFlowView },
    { path: "/settings", name: "settings", component: SettingsView }
  ]
});
