<template>
  <n-config-provider :theme="null">
    <n-message-provider>
      <n-dialog-provider>
        <n-notification-provider>
          <div id="app">
            <n-layout>
              <n-layout-header bordered style="padding: 0 24px; height: 64px; display: flex; align-items: center;">
                <div style="display: flex; align-items: center; width: 100%;">
                  <h2 style="margin: 0; margin-right: auto;">
                    <router-link to="/" style="text-decoration: none; color: inherit;">
                      📚 Panicless Library
                    </router-link>
                  </h2>
                  <n-space v-if="isAuthenticated">
                    <n-button text tag="a" @click="$router.push('/dashboard')">Dashboard</n-button>
                    <n-button text tag="a" @click="$router.push('/books')">Books</n-button>
                    <n-button text tag="a" @click="$router.push('/readings')">Readings</n-button>
                    <n-button text tag="a" @click="handleLogout">Logout</n-button>
                  </n-space>
                  <n-space v-else>
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
        </n-notification-provider>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup>
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/store/auth'
import {
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
  NNotificationProvider,
  NLayout,
  NLayoutHeader,
  NLayoutContent,
  NButton,
  NSpace
} from 'naive-ui'

const authStore = useAuthStore()
const router = useRouter()

const isAuthenticated = computed(() => authStore.isAuthenticated)

const handleLogout = () => {
  authStore.logout()
  router.push('/login')
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
