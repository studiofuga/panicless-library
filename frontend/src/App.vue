<template>
  <n-config-provider :theme="null">
    <n-message-provider>
      <n-dialog-provider>
        <n-notification-provider>
          <!-- Public layout (no sidebar) -->
          <template v-if="isPublicRoute || !isAuthenticated">
            <div id="app">
              <n-layout>
                <n-layout-header bordered style="padding: 0 24px; height: 64px; display: flex; align-items: center;">
                  <div style="display: flex; align-items: center; width: 100%;">
                    <h2 style="margin: 0; margin-right: auto;">
                      <router-link to="/" style="text-decoration: none; color: inherit;">
                        Panicless Library
                      </router-link>
                    </h2>
                    <n-space v-if="!isAuthenticated">
                      <n-button text tag="a" @click="$router.push('/login')">Login</n-button>
                      <n-button text tag="a" @click="$router.push('/register')">Register</n-button>
                    </n-space>
                  </div>
                </n-layout-header>
                <n-layout-content style="padding: 24px;">
                  <router-view />
                </n-layout-content>
              </n-layout>
            </div>
          </template>

          <!-- Authenticated layout (with sidebar) -->
          <template v-else>
            <div id="app">
              <n-layout has-sider style="height: 100vh;">
                <n-layout-sider
                  bordered
                  show-trigger
                  collapse-mode="width"
                  :collapsed-width="64"
                  :width="240"
                  :collapsed="collapsed"
                  @collapse="collapsed = true"
                  @expand="collapsed = false"
                  :native-scrollbar="false"
                  style="height: 100vh;"
                >
                  <div style="padding: 16px; text-align: center; font-weight: 700; font-size: 16px; white-space: nowrap; overflow: hidden;">
                    {{ collapsed ? 'PL' : 'Panicless Library' }}
                  </div>
                  <n-menu
                    :collapsed="collapsed"
                    :collapsed-width="64"
                    :collapsed-icon-size="22"
                    :options="menuOptions"
                    :value="activeMenuKey"
                    @update:value="handleMenuSelect"
                  />
                </n-layout-sider>
                <n-layout>
                  <n-layout-content content-style="padding: 24px;" :native-scrollbar="false">
                    <router-view />
                  </n-layout-content>
                </n-layout>
              </n-layout>
            </div>
          </template>
        </n-notification-provider>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup>
import { ref, computed, h } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { storeToRefs } from 'pinia'
import { useAuthStore } from '@/store/auth'
import {
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
  NNotificationProvider,
  NLayout,
  NLayoutHeader,
  NLayoutContent,
  NLayoutSider,
  NMenu,
  NButton,
  NSpace
} from 'naive-ui'

const authStore = useAuthStore()
const router = useRouter()
const route = useRoute()

const { isAuthenticated, isAdmin } = storeToRefs(authStore)
const collapsed = ref(false)

const isPublicRoute = computed(() => {
  return route.meta?.public === true
})

const activeMenuKey = computed(() => {
  const path = route.path
  if (path.startsWith('/statistics')) return 'statistics'
  if (path.startsWith('/books')) return 'books'
  if (path.startsWith('/readings')) return 'readings'
  if (path.startsWith('/settings')) return 'settings'
  if (path.startsWith('/admin')) return 'admin'
  return null
})

const menuOptions = computed(() => {
  const options = [
    {
      label: 'Statistics',
      key: 'statistics'
    },
    {
      label: 'Books',
      key: 'books'
    },
    {
      label: 'Readings',
      key: 'readings'
    },
    {
      type: 'divider',
      key: 'd1'
    },
    {
      label: 'Settings',
      key: 'settings'
    }
  ]

  if (isAdmin.value) {
    options.push({
      label: 'Admin',
      key: 'admin'
    })
  }

  options.push(
    {
      type: 'divider',
      key: 'd2'
    },
    {
      label: 'Logout',
      key: 'logout'
    }
  )

  return options
})

const handleMenuSelect = (key) => {
  if (key === 'logout') {
    authStore.logout()
    router.push('/login')
    return
  }

  const routes = {
    statistics: '/statistics',
    books: '/books',
    readings: '/readings',
    settings: '/settings',
    admin: '/admin/users'
  }

  if (routes[key]) {
    router.push(routes[key])
  }
}
</script>

<style>
#app {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

a {
  color: #18a058;
}
</style>
