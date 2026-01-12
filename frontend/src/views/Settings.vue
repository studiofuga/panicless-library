<template>
  <div class="settings-container">
    <n-card>
      <template #header>
        <div class="header-content">
          <span>⚙️ Settings</span>
        </div>
      </template>

      <!-- Tabs for different settings sections -->
      <n-tabs type="line" animated>
        <!-- Profile Settings Tab -->
        <n-tab-pane name="profile" tab="Profile">
          <div class="tab-content">
            <n-space vertical :size="24">
              <div>
                <h3 style="margin: 0 0 16px 0">User Information</h3>
                <n-descriptions :columns="1" border>
                  <n-descriptions-item label="Username">
                    {{ currentUser?.username }}
                  </n-descriptions-item>
                  <n-descriptions-item label="Email">
                    {{ currentUser?.email }}
                  </n-descriptions-item>
                  <n-descriptions-item label="Full Name">
                    {{ currentUser?.full_name || 'Not set' }}
                  </n-descriptions-item>
                  <n-descriptions-item label="Joined">
                    {{ formatDate(currentUser?.created_at) }}
                  </n-descriptions-item>
                </n-descriptions>
              </div>

              <n-divider />

              <div>
                <h3 style="margin: 0 0 16px 0">Account Actions</h3>
                <n-space>
                  <n-button type="warning" @click="showLogoutConfirm">
                    Logout
                  </n-button>
                </n-space>
              </div>
            </n-space>
          </div>
        </n-tab-pane>

        <!-- AI Connectors Tab -->
        <n-tab-pane name="connectors" tab="🤖 AI Connectors">
          <div class="tab-content">
            <n-space vertical :size="24">
              <!-- Introduction -->
              <n-alert type="info" :closable="false">
                <template #icon>
                  <span>ℹ️</span>
                </template>
                Add API tokens for AI providers (Anthropic, Gemini, ChatGPT) to enable integrations.
                Your tokens are encrypted and stored securely.
              </n-alert>

              <!-- Connectors List -->
              <n-space vertical :size="16" style="width: 100%">
                <!-- Anthropic Connector -->
                <ConnectorCard
                  provider="anthropic"
                  title="Anthropic Claude"
                  description="Add your Claude API key from https://console.anthropic.com"
                  icon="🧠"
                  placeholder="sk-ant-..."
                  @add="handleAddConnector"
                  @delete="handleDeleteConnector"
                  @toggle="handleToggleConnector"
                />

                <!-- Gemini Connector -->
                <ConnectorCard
                  provider="gemini"
                  title="Google Gemini"
                  description="Add your Gemini API key from https://makersuite.google.com/app/apikey"
                  icon="✨"
                  placeholder="AIza..."
                  @add="handleAddConnector"
                  @delete="handleDeleteConnector"
                  @toggle="handleToggleConnector"
                />

                <!-- ChatGPT/OpenAI Connector -->
                <ConnectorCard
                  provider="chatgpt"
                  title="OpenAI ChatGPT"
                  description="Add your OpenAI API key from https://platform.openai.com/api-keys"
                  icon="💬"
                  placeholder="sk-..."
                  @add="handleAddConnector"
                  @delete="handleDeleteConnector"
                  @toggle="handleToggleConnector"
                />
              </n-space>

              <!-- Active Connectors Summary -->
              <n-divider />
              <div v-if="activeConnectors.length > 0">
                <h4>Active Connectors ({{ activeConnectors.length }})</h4>
                <n-space :size="8">
                  <n-tag
                    v-for="connector in activeConnectors"
                    :key="connector.id"
                    type="success"
                    round
                  >
                    {{ getProviderIcon(connector.provider) }} {{ connector.provider }}
                  </n-tag>
                </n-space>
              </div>
              <div v-else>
                <n-empty description="No active connectors yet" />
              </div>
            </n-space>
          </div>
        </n-tab-pane>
      </n-tabs>
    </n-card>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useMessage } from 'naive-ui'
import { useAuthStore } from '@/store/auth'
import { useConnectorsStore } from '@/store/connectors'
import { formatDistanceToNow } from 'date-fns'
import ConnectorCard from '@/components/connectors/ConnectorCard.vue'

const router = useRouter()
const message = useMessage()
const authStore = useAuthStore()
const connectorsStore = useConnectorsStore()

// Computed
const currentUser = computed(() => authStore.user)
const activeConnectors = computed(() => connectorsStore.activeConnectors)

// Helpers
const formatDate = (dateString) => {
  if (!dateString) return 'N/A'
  const date = new Date(dateString)
  return formatDistanceToNow(date, { addSuffix: true })
}

const getProviderIcon = (provider) => {
  const icons = {
    anthropic: '🧠',
    gemini: '✨',
    chatgpt: '💬'
  }
  return icons[provider] || '🔗'
}

// Event handlers
const handleAddConnector = async ({ provider, apiToken }) => {
  try {
    await connectorsStore.createOrUpdateConnector(provider, apiToken)
    message.success(`${provider} connector added successfully`)
  } catch (error) {
    message.error(`Failed to add ${provider} connector: ${connectorsStore.error}`)
  }
}

const handleDeleteConnector = async (provider) => {
  try {
    await connectorsStore.deleteConnector(provider)
    message.success(`${provider} connector removed`)
  } catch (error) {
    message.error(`Failed to remove ${provider} connector: ${connectorsStore.error}`)
  }
}

const handleToggleConnector = async (provider) => {
  try {
    await connectorsStore.toggleConnector(provider)
    const connector = connectorsStore.getConnector(provider)
    const status = connector.is_active ? 'enabled' : 'disabled'
    message.success(`${provider} connector ${status}`)
  } catch (error) {
    message.error(`Failed to toggle ${provider} connector: ${connectorsStore.error}`)
  }
}

const showLogoutConfirm = () => {
  window.$dialog.create({
    title: 'Logout',
    content: 'Are you sure you want to logout?',
    positiveText: 'Logout',
    negativeText: 'Cancel',
    onPositiveClick: () => {
      authStore.logout()
      router.push('/login')
      message.success('Logged out successfully')
    }
  })
}

// Load connectors on mount
import { onMounted } from 'vue'
onMounted(async () => {
  try {
    await connectorsStore.fetchConnectors()
  } catch (error) {
    message.error('Failed to load connectors')
  }
})
</script>

<style scoped>
.settings-container {
  max-width: 900px;
  margin: 0 auto;
  padding: 24px;
}

.header-content {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 18px;
  font-weight: 600;
}

.tab-content {
  padding: 24px;
}

h3 {
  color: #333;
  font-size: 16px;
}

h4 {
  color: #666;
  font-size: 14px;
  margin: 0 0 12px 0;
}
</style>
