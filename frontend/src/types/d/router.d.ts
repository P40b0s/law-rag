// This can be directly added to any of your `.ts` files like `router.ts`
// It can also be added to a `.d.ts` file. Make sure it's included in
// project's tsconfig.json "files"
import { Role } from '@/types/user_role'
import 'vue-router'
import {RouteMeta} from 'vue-router'
import {type RouteRecordNameGeneric} from 'vue-router'
import {RouteName} from '@/types/route_name'
import { Privilegy } from '../permission'
// To ensure it is treated as a module, add at least one `export` statement
export {}

declare module 'vue-router' 
{
  export interface RouteMeta 
  {
    // is optional
    isAdmin?: boolean
    // must be declared by every route
    requiresAuth: boolean,
    title: string,
    roles: Role[],
    privilegies: Privilegy[]
  }
  export type RouteRecordNameGeneric = RouteName;
}