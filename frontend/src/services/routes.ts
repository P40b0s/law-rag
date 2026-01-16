import { NIcon } from "naive-ui";
import router from "../router";
import { Component, computed, type ComputedRef, h, VNode } from "vue";
import { RouteMeta } from "vue-router";
import { Role } from "../types/user_role";
import { Permission } from "../types/permission";
import {
  UserAvatar as ProfileIcon,
  ChartLineData as StatisticIcon,
  UserSettings as SettingsIcon,
  DocumentView as DocumentIcon,
  MailAll as PacketsIcon,
  Login as LoginIcon,
  DataBaseAlt as DictIcon,
  UserMultiple,
  ChartHistogram,
  CalendarHeatMap as CalendarIcon,
  Apps as AppsIcon,
  TaskView
} from '@vicons/carbon'
import Login from '@/views/Login.vue'
import Documents from '@/views/Documents.vue'
import Reports from "@/views/Reports.vue";
import Profile from "@/views/Profile.vue";
import Users from '@/views/Users.vue';
import useUser from "@/composables/useUser";
import { title } from 'process';
import MenuUserIcon from "@/components/MenuUserIcon.vue";
import Packets from "@/views/Packets.vue";
import useVisible from "@/composables/useVisible";
import Dictionaries from "@/views/Dictionaries.vue";
import Employees from "@/views/Employees.vue";
import Dashboard from "@/views/Dashboard.vue";
import Statistic from "@/views/Statistic.vue";
import Calendar from "@/views/Calendar.vue";
import Tasks from "@/views/Tasks.vue";

const {visible} = useVisible();
export enum RouteName
{
  Documents = "documents",
  Dashboard = "dashboard",
  Statistic = "statistic",
  Calendar = "calendar",
  Login = "login",
  Profile = "profile",
  Reports = "reports",
  Users = "users",
  Packets = "packets",
  Dictionaries = "dictionaries",
  Employees = "employees",
  Tasks = "tasks"
}


type MenuItem = {
  title: string,
  icon: Component,
  placement: 'up' | 'down'
}
class Routing
{
  
  name: RouteName;
  title: string;
  roles: Role[];
  permissions: Permission[]
  reques_auth: boolean;
  path: string;
  component: VNode;
  menu?: MenuItem;
   /**
     * Отображает иконку в меню если текущая роль юзера подходит для этого маршрута
     * 
     */
  visible: ComputedRef<boolean>;

  constructor(name: RouteName, path: string, title: string, component: VNode)
  {
    this.name = name;
    this.title = title;
    this.roles = [];
    this.permissions = [];
    this.reques_auth = false;
    this.path = path;
    this.component = component;
    this.visible = visible(this.roles, this.permissions);
  }
   
  with_menu(title: string, icon: Component, placement: 'up' | 'down' = 'up'): Routing
  {
    this.menu = {
      title,
      icon,
      placement
    };
    return this;
  }
  with_roles(roles: Role[]): Routing
  {
    this.roles = roles;
    this.reques_auth = roles.length > 0;
    this.visible = visible(this.roles, this.permissions);
    return this;
  }
  with_privilegies(permissions: Permission[]): Routing
  {
    this.permissions = permissions;
    this.visible = visible(this.roles, this.permissions);
    return this;
  }
  get_meta(): RouteMeta
  {
    return {
      	requiresAuth: this.reques_auth,
        title: this.title,
        roles: this.roles,
        privilegies: this.permissions
    } as RouteMeta
  }
  get_path()
  {
    return this.path;
  }
  get_component()
  {
    return this.component;
  }
  get_name()
  {
    return this.name;
  }
  render_menu_label()
    {
        return h('div',
            {
                style:
                {
                    fontSize: "16px",
                    background: "transparent",
                    padding: "2px"
                },
                onClick:()=>
                {
                    router.push({name: this.name})
                }
            },
             this.menu?.title
        )
    }
    render_menu_icon() 
    {
        return () => h(NIcon, 
        {
          color: router.currentRoute.value.name == this.name ? 'rgb(146,230,26)' : 'rgb(139,140,115)',
          onClick:()=>
          {
            router.push({name: this.name})
          }
        }, 
        { 
            default: () => h(this.menu?.icon ?? 'span') 
        })
    }
  

    //menu_visible = computed(() => 
    //{
    //  if(!this.menu)
    //    return false;
      // const {get_role} = useUser();
      // if(this.roles.length == 0)
      //   return true
      // const current_role = get_role();
      // if(current_role == 'Administrator')
      //  return true;
      // if(this.roles.includes(current_role))
      //   return true;
      // else
      //   return false
    //})
}

const routes = new Map<RouteName, Routing>([
  
    [RouteName.Dashboard, new Routing(
      RouteName.Dashboard,
      '/dashboard',
      'Статус',
      h(Dashboard))
      .with_menu('Статус', AppsIcon)
      .with_roles(['Administrator', 'User'])
    ],
    [RouteName.Dictionaries, new Routing(
      RouteName.Dictionaries,
      '/dictionaries',
      'Словари',
      h(Dictionaries))
      .with_menu('Словари', DictIcon)
      .with_roles(['Administrator', 'User'])
    ],
    [RouteName.Employees, new Routing(
      RouteName.Employees,
      '/employees',
      'Сотрудники',
      h(Employees))
      .with_menu('Сотрудники', UserMultiple)
      .with_roles(['Administrator', 'User'])
    ],
    [RouteName.Calendar, new Routing(
      RouteName.Calendar,
      '/calendar',
      'Календарь',
      h(Calendar))
      .with_menu('Календарь', CalendarIcon)
      .with_roles(['Administrator', 'User'])
    ],
    [RouteName.Statistic, new Routing(
      RouteName.Statistic,
      '/statistic',
      'Статистика',
      h(Statistic))
      .with_menu('Статистика', ChartHistogram)
      .with_roles(['Administrator', 'User'])
    ],
    [RouteName.Tasks, new Routing(
      RouteName.Tasks,
      '/tasks',
      'Задачи',
      h(Tasks))
      .with_menu('Задачи', TaskView)
      .with_roles(['Administrator', 'User'])
    ],
    [RouteName.Login, new Routing(
      RouteName.Login,
      '/login',
      'Страница входа',
      h(Login)
    )],
    [RouteName.Profile, new Routing(
      RouteName.Profile,
      '/profile',
      'Профиль',
      h(Profile))
      .with_menu('Профиль', MenuUserIcon, 'down')
      .with_roles(['Administrator', 'User'])
    ],
    [RouteName.Users, new Routing(
      RouteName.Users,
      '/users',
      'Список пользователей',
      h(Users))
      .with_menu('Список пользователей', SettingsIcon)
      .with_roles(['Administrator'])
    ],
  ]
)
export {routes};
