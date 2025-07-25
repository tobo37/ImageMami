import { createRouter, createWebHashHistory, RouteRecordRaw } from 'vue-router';

import Duplicate from '../views/Duplicate.vue';
import ImportView from '../views/Import.vue';
import SortView from '../views/Sort.vue';
import Blackhole from '../views/Blackhole.vue';
import Home from '../views/Home.vue';
import ConfigView from '../views/Config.vue';

const routes: Array<RouteRecordRaw> = [
  { path: '/', name: 'home', component: Home },
  { path: '/duplicate', name: 'duplicate', component: Duplicate },
  { path: '/import', name: 'import', component: ImportView },
  { path: '/sort', name: 'sort', component: SortView },
  { path: '/blackhole', name: 'blackhole', component: Blackhole },
  { path: '/config', name: 'config', component: ConfigView },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
