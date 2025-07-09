import { createRouter, createWebHashHistory, RouteRecordRaw } from 'vue-router'

import Duplicate from '../components/Duplicate.vue'
import ImportView from '../components/Import.vue'
import SortView from '../components/Sort.vue'
import Blackhole from '../components/Blackhole.vue'
import TrainingData from '../components/TrainingData.vue'
import Home from '../views/Home.vue'

const routes: Array<RouteRecordRaw> = [
  { path: '/', name: 'home', component: Home },
  { path: '/duplicate', name: 'duplicate', component: Duplicate },
  { path: '/import', name: 'import', component: ImportView },
  { path: '/sort', name: 'sort', component: SortView },
  { path: '/blackhole', name: 'blackhole', component: Blackhole },
  { path: '/training', name: 'training', component: TrainingData },
]

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

export default router
