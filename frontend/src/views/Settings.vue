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
                  <n-descriptions-item label="Role">
                    <n-tag :type="currentUser?.role?.toLowerCase() === 'admin' ? 'warning' : 'info'" size="small">
                      {{ currentUser?.role }}
                    </n-tag>
                  </n-descriptions-item>
                  <n-descriptions-item label="Status">
                    <n-tag :type="currentUser?.enabled ? 'success' : 'error'" size="small">
                      {{ currentUser?.enabled ? 'Active' : 'Disabled' }}
                    </n-tag>
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

              <n-divider />

              <div>
                <h3 style="margin: 0 0 16px 0">Change Password</h3>
                <n-form ref="passwordFormRef" :model="passwordForm" :rules="passwordRules" style="max-width: 400px;">
                  <n-form-item label="Current Password" path="current_password">
                    <n-input v-model:value="passwordForm.current_password" type="password" placeholder="Enter current password" />
                  </n-form-item>
                  <n-form-item label="New Password" path="new_password">
                    <n-input v-model:value="passwordForm.new_password" type="password" placeholder="Enter new password (min 8 characters)" />
                  </n-form-item>
                  <n-form-item label="Confirm New Password" path="confirm_new_password">
                    <n-input v-model:value="passwordForm.confirm_new_password" type="password" placeholder="Confirm new password" />
                  </n-form-item>
                  <n-button type="primary" :loading="changingPassword" @click="handleChangePassword">
                    Change Password
                  </n-button>
                </n-form>
              </div>

              <n-divider />

              <div>
                <h3 style="margin: 0 0 16px 0">Version Info</h3>
                <n-descriptions :columns="1" border>
                  <n-descriptions-item label="Frontend">
                    {{ frontendVersion }}
                  </n-descriptions-item>
                  <n-descriptions-item label="Backend">
                    <n-spin v-if="loadingBackendVersion" :size="16" />
                    <template v-else>{{ backendVersion }}</template>
                  </n-descriptions-item>
                </n-descriptions>
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

        <!-- Import / Export Tab -->
        <n-tab-pane name="import-export" tab="Import / Export">
          <div class="tab-content">
            <n-space vertical :size="24">
              <!-- Import Section -->
              <div>
                <h3 style="margin: 0 0 16px 0">Import from Goodreads</h3>
                <n-alert type="info" title="How to export from Goodreads" style="margin-bottom: 16px;">
                  <ol style="margin: 8px 0 0 0; padding-left: 20px;">
                    <li>Go to Goodreads and navigate to "My Books"</li>
                    <li>Click "Import and export" at the top</li>
                    <li>Click "Export Library" to download your CSV file</li>
                    <li>Upload the downloaded CSV file below</li>
                  </ol>
                </n-alert>

                <n-upload
                  :custom-request="handleUpload"
                  :show-file-list="true"
                  :max="1"
                  accept=".csv"
                  @change="handleFileChange"
                  :disabled="importing"
                >
                  <n-button :disabled="importing">
                    Select CSV File
                  </n-button>
                </n-upload>

                <n-space v-if="selectedFile" style="margin-top: 12px;">
                  <n-tag type="success">
                    Selected: {{ selectedFile.name }} ({{ formatFileSize(selectedFile.size) }})
                  </n-tag>
                </n-space>

                <n-button
                  v-if="selectedFile"
                  type="primary"
                  size="large"
                  :loading="importing"
                  @click="handleImport"
                  block
                  style="margin-top: 12px;"
                >
                  {{ importing ? 'Importing...' : 'Import Books' }}
                </n-button>

                <!-- Import Progress / Result -->
                <div v-if="importing || importResult" style="margin-top: 16px;">
                  <n-spin v-if="importing" size="large">
                    <template #description>
                      Importing your books... This may take a moment.
                    </template>
                  </n-spin>

                  <div v-if="importResult && !importing">
                    <n-alert
                      :type="importResult.summary.failed_imports === 0 ? 'success' : 'warning'"
                      :title="importResult.summary.failed_imports === 0 ? 'Import Completed Successfully!' : 'Import Completed with Some Errors'"
                      style="margin-bottom: 16px;"
                    >
                      <n-space>
                        <n-statistic label="Total Rows" :value="importResult.summary.total_rows" />
                        <n-statistic label="Successful" :value="importResult.summary.successful_imports" />
                        <n-statistic label="Failed" :value="importResult.summary.failed_imports" />
                        <n-statistic label="Books Created" :value="importResult.summary.books_created" />
                        <n-statistic label="Books Updated" :value="importResult.summary.books_updated" />
                        <n-statistic label="Readings Created" :value="importResult.summary.readings_created" />
                      </n-space>
                    </n-alert>

                    <n-card v-if="importResult.errors.length > 0" title="Errors" size="small" :bordered="false">
                      <n-list bordered>
                        <n-list-item v-for="error in importResult.errors" :key="error.row_number">
                          <n-thing>
                            <template #header>
                              Row {{ error.row_number }}
                              <span v-if="error.book_title"> - {{ error.book_title }}</span>
                            </template>
                            <template #description>
                              <n-text type="error">{{ error.error }}</n-text>
                            </template>
                          </n-thing>
                        </n-list-item>
                      </n-list>
                    </n-card>

                    <n-space justify="end" style="margin-top: 12px;">
                      <n-button @click="resetImport">Import Another File</n-button>
                    </n-space>
                  </div>
                </div>
              </div>

              <n-divider />

              <!-- Export Section -->
              <div>
                <h3 style="margin: 0 0 16px 0">Export</h3>
                <n-alert type="info" :closable="false">
                  Export functionality is coming soon.
                </n-alert>
                <n-button disabled style="margin-top: 12px;">
                  Export Library (Coming Soon)
                </n-button>
              </div>
            </n-space>
          </div>
        </n-tab-pane>

        <!-- Danger Zone Tab -->
        <n-tab-pane name="danger-zone" tab="Danger Zone">
          <div class="tab-content">
            <n-alert type="error" :closable="false" style="margin-bottom: 24px;">
              Actions in this section are irreversible. Please proceed with caution.
            </n-alert>

            <n-card style="border: 1px solid #e88080;">
              <h3 style="margin: 0 0 12px 0; color: #d03050;">Delete All Data</h3>
              <p style="margin: 0 0 16px 0; color: #666;">
                This will permanently delete all your books and reading records. This action cannot be undone.
              </p>
              <n-button type="error" @click="handleDeleteAllData" :loading="deletingAllData">
                Delete All My Data
              </n-button>
            </n-card>
          </div>
        </n-tab-pane>
      </n-tabs>
    </n-card>
  </div>
</template>

<script setup>
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useMessage, useDialog } from 'naive-ui'
import { useAuthStore } from '@/store/auth'
import { useBooksStore } from '@/store/books'
import { useConnectorsStore } from '@/store/connectors'
import { formatDistanceToNow } from 'date-fns'
import ConnectorCard from '@/components/connectors/ConnectorCard.vue'
import apiClient from '@/api/client'

const router = useRouter()
const message = useMessage()
const dialog = useDialog()
const authStore = useAuthStore()
const booksStore = useBooksStore()
const connectorsStore = useConnectorsStore()

// Version info
const frontendVersion = __APP_VERSION__
const backendVersion = ref('...')
const loadingBackendVersion = ref(true)

// Computed
const currentUser = computed(() => authStore.user)
const activeConnectors = computed(() => connectorsStore.activeConnectors)

// Change password
const passwordFormRef = ref(null)
const changingPassword = ref(false)
const passwordForm = ref({
  current_password: '',
  new_password: '',
  confirm_new_password: ''
})

const passwordRules = {
  current_password: [
    { required: true, message: 'Current password is required', trigger: 'blur' }
  ],
  new_password: [
    { required: true, message: 'New password is required', trigger: 'blur' },
    { min: 8, message: 'Password must be at least 8 characters', trigger: 'blur' }
  ],
  confirm_new_password: [
    { required: true, message: 'Please confirm your new password', trigger: 'blur' },
    {
      validator: (rule, value) => value === passwordForm.value.new_password,
      message: 'Passwords do not match',
      trigger: 'blur'
    }
  ]
}

const handleChangePassword = async () => {
  try {
    await passwordFormRef.value?.validate()
    changingPassword.value = true
    await authStore.changePassword(passwordForm.value.current_password, passwordForm.value.new_password)
    message.success('Password changed successfully')
    passwordForm.value = { current_password: '', new_password: '', confirm_new_password: '' }
  } catch (error) {
    if (error.response) {
      message.error(error.response.data.message || 'Failed to change password')
    } else if (error.errors) {
      return
    } else {
      message.error('Failed to change password')
    }
  } finally {
    changingPassword.value = false
  }
}

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

// Import functionality
const selectedFile = ref(null)
const importing = ref(false)
const importResult = ref(null)

function handleFileChange(options) {
  if (options.fileList.length > 0) {
    selectedFile.value = options.fileList[0].file
  } else {
    selectedFile.value = null
  }
}

function handleUpload({ file, onFinish }) {
  selectedFile.value = file.file
  onFinish()
}

async function handleImport() {
  if (!selectedFile.value) {
    message.error('Please select a CSV file first')
    return
  }
  if (!selectedFile.value.name.endsWith('.csv')) {
    message.error('Please select a valid CSV file')
    return
  }
  if (selectedFile.value.size > 10 * 1024 * 1024) {
    message.error('File size must be less than 10MB')
    return
  }

  importing.value = true
  importResult.value = null

  try {
    const result = await booksStore.importGoodreadsCSV(selectedFile.value)
    importResult.value = result
    if (result.summary.failed_imports === 0) {
      message.success(`Successfully imported ${result.summary.successful_imports} books!`)
    } else {
      message.warning(`Imported ${result.summary.successful_imports} books with ${result.summary.failed_imports} errors`)
    }
  } catch (error) {
    message.error(error.response?.data?.error || 'Failed to import CSV file')
  } finally {
    importing.value = false
  }
}

function resetImport() {
  selectedFile.value = null
  importResult.value = null
}

function formatFileSize(bytes) {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(2) + ' MB'
}

// Danger Zone
const deletingAllData = ref(false)

const handleDeleteAllData = () => {
  dialog.error({
    title: 'Delete All Data',
    content: 'Are you absolutely sure? This will permanently delete ALL your books and reading records. This action CANNOT be undone.',
    positiveText: 'Yes, Delete Everything',
    negativeText: 'Cancel',
    onPositiveClick: async () => {
      try {
        deletingAllData.value = true
        const res = await apiClient.delete('/api/books/all')
        message.success(`Deleted ${res.data.books_deleted} books and ${res.data.readings_deleted} readings.`)
      } catch (error) {
        message.error(error.response?.data?.error || 'Failed to delete data')
      } finally {
        deletingAllData.value = false
      }
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

  try {
    const res = await apiClient.get('/api/version')
    backendVersion.value = res.data.version
  } catch {
    backendVersion.value = 'N/A'
  } finally {
    loadingBackendVersion.value = false
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
