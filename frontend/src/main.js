import { createApp } from 'vue'
import { createPinia } from 'pinia'
import naive from 'naive-ui'
import App from './App.vue'
import router from './router'
import { initializeApiClient } from './api/client'

async function bootstrap() {
  // Load runtime configuration before initializing the app
  await initializeApiClient()

  const app = createApp(App)
  const pinia = createPinia()

  app.use(pinia)
  app.use(router)
  app.use(naive)

  app.mount('#app')
}

bootstrap()
