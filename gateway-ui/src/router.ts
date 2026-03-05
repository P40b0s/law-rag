import { createRouter, createWebHistory } from 'vue-router'
import LoginView from './views/LoginView.vue'
import AdminView from './views/AdminView.vue'
import GuestServicesView from './views/GuestServicesView.vue'
import { get_stored_token } from './services/api'

const router = createRouter({
    history: createWebHistory('/ui/'),
    routes: [
        {
            path: '/login',
            name: 'login',
            component: LoginView,
            meta: { requiresAuth: false },
        },
        {
            path: '/admin',
            name: 'admin',
            component: AdminView,
            meta: { requiresAuth: true },
        },
        {
            path: '/services',
            name: 'services',
            component: GuestServicesView,
            meta: { requiresAuth: false },
        },
        {
            path: '/',
            redirect: { name: 'login' },
        },
    ],
})

router.beforeEach((to) => {
    const authenticated = !!get_stored_token()
    if (to.meta.requiresAuth && !authenticated) {
        return { name: 'login' }
    }
    if (to.name === 'login' && authenticated) {
        return { name: 'admin' }
    }
})

export default router
