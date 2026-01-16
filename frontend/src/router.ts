import { createMemoryHistory, createRouter, createWebHistory, type RouteRecordRaw, RouterLink } from 'vue-router'

import LoginView from './components/Login.vue'
import { Component, type CSSProperties, h, type RendererElement, type RendererNode, type VNode } from 'vue'
import {NButton, NIcon} from 'naive-ui';
//import { DocumentsList } from './components/documets_list/documents_list.tsx';
//import { RedactionsList } from './components/redactions_list/redactions_list.tsx';
//import { user_store } from './store/index.ts';
//import { type UserRole } from './models/user.ts';
//import user_service from './services/user_service.ts';
import DocumentComparator from './components/document_comparator/DocumentComparator.vue';
import { Role } from './types/user_role';
import Login from './views/Login.vue';
import { is_authentificated } from './composables/useUser';
import Main from './views/Main.vue';
import {type RouteRecordNameGeneric, RouteMeta} from 'vue-router'
import { RouteName, routes } from './services/routes';
import { Permission } from './types/permission';

const router_routes: RouteRecordRaw[] = Array.from(routes.values()).map(route => ({
  path: route.get_path(),
  name: route.get_name(),
  component: route.get_component(),
  meta: route.get_meta()
}))

const router = createRouter({
  history: createWebHistory(),
  routes: [...router_routes, 
    {
      path: '/',
      redirect: {name: RouteName.Employees}
    },
  ]
  // [
  //   {
  //     path: '/',
  //     redirect: {name: RouteName.Documents}
  //   },
  //   {
  //     path: routes.documents.get_path(),
  //     name: routes.documents.get_name(),
  //     component: routes.documents.get_component(),
  //     meta: routes.documents.get_meta()
  //   },
  //   {
  //     path: routes.login.get_path(),
  //     name: routes.login.get_name(),
  //     component: routes.login.get_component(),
  //     meta: routes.login.get_meta()
  //   },
  //   {
  //     path: routes.profile.get_path(),
  //     name: routes.profile.get_name(),
  //     component: routes.profile.get_component(),
  //     meta: routes.profile.get_meta()
  //   },
  //   {
  //     path: routes.reports.get_path(),
  //     name: routes.reports.get_name(),
  //     component: routes.reports.get_component(),
  //     meta: routes.reports.get_meta()
  //   },
  // ]
})

router.afterEach((to, from) => 
{
  const meta = to.meta as RouteMeta;
  //Vue.nextTick(() => 
  //{
      document.title = meta.title;
  //})
})
router.beforeEach(async (to, from, next) => 
{
  const meta = to.meta
  if (meta.requiresAuth)
  {
    if(is_authentificated.value)
    {
      next();
    }
    else
    {
      if(router.currentRoute.value.name != RouteName.Login)
        next({name: RouteName.Login})
    } 
  }
  else next()
})
export default router;