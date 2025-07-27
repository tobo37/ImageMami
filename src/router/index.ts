import { createRouter, createWebHashHistory, RouteRecordRaw } from 'vue-router';

import Home from '../views/Home.vue';

const routes: Array<RouteRecordRaw> = [
  { path: '/', name: 'home', component: Home },
  {
    path: '/duplicate',
    name: 'duplicate',
    component: () => import('../views/Duplicate.vue'),
    meta: { titleKey: 'duplicate.title' },
  },
  {
    path: '/import',
    name: 'import',
    component: () => import('../views/Import.vue'),
    meta: { titleKey: 'import.title' },
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
