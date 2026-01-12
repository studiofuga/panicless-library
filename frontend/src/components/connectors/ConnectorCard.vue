<template>
  <n-card :bordered="true" :segmented="false" class="connector-card">
    <template #header>
      <div class="card-header">
        <span class="provider-icon">{{ icon }}</span>
        <div class="provider-info">
          <h4>{{ title }}</h4>
          <p>{{ description }}</p>
        </div>
        <div class="status-badge" v-if="connector">
          <n-badge
            :type="connector.is_active ? 'success' : 'default'"
            :value="connector.is_active ? 'Active' : 'Inactive'"
          />
        </div>
      </div>
    </template>

    <template v-if="connector && !isEditing">
      <!-- Connector exists - show details -->
      <n-space vertical :size="16">
        <div class="connector-details">
          <n-descriptions :columns="1" :bordered="false" size="small">
            <n-descriptions-item label="Status">
              <n-switch
                :value="connector.is_active"
                @update:value="handleToggle"
                :loading="isTogglingLoading"
              />
            </n-descriptions-item>
            <n-descriptions-item label="Added">
              {{ formatDate(connector.created_at) }}
            </n-descriptions-item>
            <n-descriptions-item v-if="connector.last_used_at" label="Last Used">
              {{ formatDate(connector.last_used_at) }}
            </n-descriptions-item>
            <n-descriptions-item v-else label="Last Used">
              Never
            </n-descriptions-item>
          </n-descriptions>
        </div>

        <n-space>
          <n-button
            text
            type="primary"
            size="small"
            @click="isEditing = true"
          >
            ✏️ Update Token
          </n-button>
          <n-button
            text
            type="error"
            size="small"
            @click="handleDelete"
            :loading="isDeletingLoading"
          >
            🗑️ Remove
          </n-button>
        </n-space>
      </n-space>
    </template>

    <template v-else-if="isEditing">
      <!-- Edit/Add mode -->
      <n-space vertical :size="16">
        <n-input
          v-model:value="tokenInput"
          type="password"
          :placeholder="placeholder"
          show-password-on="click"
          :loading="isLoadingAdd"
        />

        <n-space>
          <n-button
            type="primary"
            @click="handleAdd"
            :loading="isLoadingAdd"
            :disabled="!tokenInput.trim()"
          >
            💾 Save Token
          </n-button>
          <n-button
            @click="cancelEdit"
            :disabled="isLoadingAdd"
          >
            Cancel
          </n-button>
        </n-space>

        <n-alert type="warning" :closable="false" v-if="connector">
          <template #icon>
            <span>⚠️</span>
          </template>
          Updating this token will replace the existing one.
        </n-alert>

        <n-alert type="info" :closable="false" v-else>
          <template #icon>
            <span>ℹ️</span>
          </template>
          Your token will be encrypted and stored securely.
        </n-alert>
      </n-space>
    </template>

    <template v-else>
      <!-- No connector - show add button -->
      <n-empty description="Not configured" size="small">
        <template #extra>
          <n-button type="primary" size="small" @click="startAdd">
            ➕ Add {{ provider }} Token
          </n-button>
        </template>
      </n-empty>
    </template>
  </n-card>
</template>

<script setup>
import { ref, computed } from 'vue'
import { formatDistanceToNow } from 'date-fns'
import { useConnectorsStore } from '@/store/connectors'

const props = defineProps({
  provider: {
    type: String,
    required: true
  },
  title: {
    type: String,
    required: true
  },
  description: {
    type: String,
    required: true
  },
  icon: {
    type: String,
    required: true
  },
  placeholder: {
    type: String,
    default: 'Paste your API token here...'
  }
})

const emit = defineEmits(['add', 'delete', 'toggle'])

// Store
const connectorsStore = useConnectorsStore()

// State
const isEditing = ref(false)
const tokenInput = ref('')
const isLoadingAdd = ref(false)
const isDeletingLoading = ref(false)
const isTogglingLoading = ref(false)

// Computed
const connector = computed(() => connectorsStore.getConnector(props.provider))

// Methods
const formatDate = (dateString) => {
  if (!dateString) return 'N/A'
  try {
    return formatDistanceToNow(new Date(dateString), { addSuffix: true })
  } catch {
    return 'N/A'
  }
}

const startAdd = () => {
  isEditing.value = true
  tokenInput.value = ''
}

const cancelEdit = () => {
  isEditing.value = false
  tokenInput.value = ''
}

const handleAdd = async () => {
  if (!tokenInput.value.trim()) return

  isLoadingAdd.value = true
  try {
    await emit('add', {
      provider: props.provider,
      apiToken: tokenInput.value
    })
    cancelEdit()
  } finally {
    isLoadingAdd.value = false
  }
}

const handleDelete = async () => {
  window.$dialog.create({
    title: `Remove ${props.provider}?`,
    content: `Are you sure you want to remove your ${props.provider} connector? This action cannot be undone.`,
    positiveText: 'Remove',
    negativeText: 'Cancel',
    onPositiveClick: async () => {
      isDeletingLoading.value = true
      try {
        await emit('delete', props.provider)
      } finally {
        isDeletingLoading.value = false
      }
    }
  })
}

const handleToggle = async (value) => {
  isTogglingLoading.value = true
  try {
    await emit('toggle', props.provider)
  } finally {
    isTogglingLoading.value = false
  }
}
</script>

<style scoped>
.connector-card {
  transition: all 0.3s ease;
}

.connector-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.card-header {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  width: 100%;
  padding: 8px 0;
}

.provider-icon {
  font-size: 32px;
  min-width: 40px;
  text-align: center;
}

.provider-info {
  flex: 1;
}

.provider-info h4 {
  margin: 0 0 4px 0;
  font-size: 16px;
  font-weight: 600;
  color: #333;
}

.provider-info p {
  margin: 0;
  font-size: 13px;
  color: #666;
}

.status-badge {
  display: flex;
  align-items: center;
}

.connector-details {
  background-color: #f5f5f5;
  padding: 12px;
  border-radius: 4px;
}
</style>
