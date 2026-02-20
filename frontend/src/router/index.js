import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/store/auth'

const routes = [
  {
    path: '/',
    name: 'Home',
    component: () => import('@/views/Home.vue'),
    meta: { requiresAuth: false, public: true }
  },
  {
    path: '/login',
    name: 'Login',
    component: () => import('@/views/Login.vue'),
    meta: { guest: true, public: true }
  },
  {
    path: '/register',
    name: 'Register',
    component: () => import('@/views/Register.vue'),
    meta: { guest: true, public: true }
  },
  {
    path: '/complete-registration',
    name: 'CompleteRegistration',
    component: () => import('@/views/CompleteRegistration.vue'),
    meta: { guest: true, public: true }
  },
  {
    path: '/statistics',
    name: 'Statistics',
    component: () => import('@/views/Statistics.vue'),
    meta: { requiresAuth: true }
  },
  {
    path: '/dashboard',
    redirect: '/statistics'
  },
  {
    path: '/books',
    name: 'Books',
    component: () => import('@/views/BookCatalog.vue'),
    meta: { requiresAuth: true }
  },
  {
    path: '/books/:id',
    name: 'BookDetail',
    component: () => import('@/views/BookDetail.vue'),
    meta: { requiresAuth: true }
  },
  {
    path: '/readings',
    name: 'Readings',
    component: () => import('@/views/ReadingTracker.vue'),
    meta: { requiresAuth: true }
  },
  {
    path: '/settings',
    name: 'Settings',
    component: () => import('@/views/Settings.vue'),
    meta: { requiresAuth: true }
  },
  {
    path: '/authorize',
    name: 'Authorize',
    component: () => import('@/views/Authorize.vue'),
    meta: { requiresAuth: true }
  },
  {
    path: '/admin/users',
    name: 'AdminUsers',
    component: () => import('@/views/AdminUsers.vue'),
    meta: { requiresAuth: true, requiresAdmin: true }
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

// Navigation guards
router.beforeEach((to, from, next) => {
  const authStore = useAuthStore()

  if (to.meta.requiresAuth && !authStore.isAuthenticated) {
    // Protected route, user not authenticated
    next({ name: 'Login', query: { redirect: to.fullPath } })
  } else if (to.meta.requiresAdmin && !authStore.isAdmin) {
    // Admin route, user not admin
    next({ name: 'Statistics' })
  } else if (to.meta.guest && authStore.isAuthenticated) {
    // Guest route (login/register), user already authenticated
    next({ name: 'Statistics' })
  } else {
    next()
  }
})

export default router
