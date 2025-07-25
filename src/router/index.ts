import { createRouter, createWebHashHistory, RouteRecordRaw } from 'vue-router';

import Duplicate from '../views/Duplicate.vue';
import ImportView from '../views/Import.vue';
import Home from '../views/Home.vue';

const routes: Array<RouteRecordRaw> = [
  { path: '/', name: 'home', component: Home },
  { path: '/duplicate', name: 'duplicate', component: Duplicate },
  { path: '/import', name: 'import', component: ImportView },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
