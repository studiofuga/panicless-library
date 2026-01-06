<template>
  <div style="max-width: 800px; margin: 0 auto; text-align: center;">
    <h1>Welcome to Panicless Library</h1>
    <p style="font-size: 18px; margin: 24px 0;">
      Your personal book collection and reading tracker.
    </p>
    <n-space justify="center" style="margin-top: 32px;">
      <n-button type="primary" size="large" @click="$router.push('/register')" v-if="!isAuthenticated">
        Get Started
      </n-button>
      <n-button type="primary" size="large" @click="$router.push('/dashboard')" v-else>
        Go to Dashboard
      </n-button>
      <n-button size="large" @click="$router.push('/login')" v-if="!isAuthenticated">
        Login
      </n-button>
    </n-space>
    <n-divider />
    <n-space vertical size="large" style="margin-top: 48px; text-align: left;">
      <div>
        <h3>📚 Organize Your Library</h3>
        <p>Catalog all your books with detailed information including title, author, ISBN, and more.</p>
      </div>
      <div>
        <h3>📖 Track Your Reading</h3>
        <p>Record when you start and finish books, rate them, and add notes about your reading experience.</p>
      </div>
      <div>
        <h3>📊 View Statistics</h3>
        <p>See insights about your reading habits including books read per year and average ratings.</p>
      </div>
      <div>
        <h3>🤖 AI Integration (MCP)</h3>
        <p>Connect your library to Claude or Gemini via MCP for intelligent book queries and recommendations.</p>
      </div>
    </n-space>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { useAuthStore } from '@/store/auth'
import { NButton, NSpace, NDivider } from 'naive-ui'

const authStore = useAuthStore()
const isAuthenticated = computed(() => authStore.isAuthenticated)
</script>
