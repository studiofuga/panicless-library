<template>
  <div style="max-width: 500px; margin: 0 auto; padding-top: 2rem;">
    <n-card title="Authorization Request">
      <!-- Error Alert -->
      <n-alert
        v-if="validationError"
        type="error"
        :title="validationError"
        style="margin-bottom: 1rem;"
      />

      <!-- Loading Spinner -->
      <n-spin :show="loading">
        <div v-if="!validationError">
          <!-- Client Information -->
          <n-descriptions label-placement="left" :column="1" bordered>
            <n-descriptions-item label="Client">
              <n-text strong>{{ clientName }}</n-text>
            </n-descriptions-item>
            <n-descriptions-item label="Client ID">
              <n-code>{{ authParams.client_id }}</n-code>
            </n-descriptions-item>
            <n-descriptions-item label="Redirect URI">
              <n-code style="word-break: break-all;">{{ authParams.redirect_uri }}</n-code>
            </n-descriptions-item>
            <n-descriptions-item v-if="scopeDescription" label="Requested Access">
              <n-text>{{ scopeDescription }}</n-text>
            </n-descriptions-item>
          </n-descriptions>

          <!-- Authorization Prompt -->
          <n-space vertical style="margin-top: 1.5rem;">
            <n-text>
              <strong>{{ clientName }}</strong> is requesting access to your Panicless Library account.
            </n-text>
            <n-text depth="3" style="font-size: 0.9em;">
              By authorizing, you allow this application to access your library data on your behalf.
            </n-text>
          </n-space>

          <!-- Action Buttons -->
          <n-space vertical style="margin-top: 2rem;">
            <n-button
              type="primary"
              :loading="loading"
              @click="handleAuthorize"
              block
              :disabled="!!validationError"
            >
              Authorize
            </n-button>
            <n-button
              @click="handleDeny"
              block
              :disabled="loading"
            >
              Deny
            </n-button>
          </n-space>
        </div>
      </n-spin>
    </n-card>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useMessage } from 'naive-ui'
import {
  NCard,
  NAlert,
  NSpin,
  NDescriptions,
  NDescriptionsItem,
  NText,
  NCode,
  NSpace,
  NButton
} from 'naive-ui'
import apiClient from '@/api/client'

const route = useRoute()
const message = useMessage()

// State
const loading = ref(false)
const validationError = ref(null)
const authParams = ref({
  response_type: '',
  client_id: '',
  redirect_uri: '',
  scope: '',
  state: '',
  code_challenge: '',
  code_challenge_method: ''
})

// Client name mapping
const CLIENT_NAMES = {
  'panicless-library': 'Panicless Library',
  'claudeai': 'Claude AI'
}

// Computed properties
const clientName = computed(() => {
  return CLIENT_NAMES[authParams.value.client_id] || authParams.value.client_id
})

const scopeDescription = computed(() => {
  const scope = authParams.value.scope
  if (!scope) return null

  const SCOPE_DESCRIPTIONS = {
    'claudeai': 'Claude AI Integration Access',
    'read': 'Read access to your library',
    'write': 'Write access to your library'
  }

  return SCOPE_DESCRIPTIONS[scope] || scope
})

// Validate URL parameters
function validateParams() {
  const params = authParams.value

  // Check required parameters
  if (!params.response_type) {
    return 'Missing required parameter: response_type'
  }

  if (params.response_type !== 'code') {
    return 'Invalid response_type. Only "code" is supported.'
  }

  if (!params.client_id) {
    return 'Missing required parameter: client_id'
  }

  if (!params.redirect_uri) {
    return 'Missing required parameter: redirect_uri'
  }

  // Validate redirect_uri format
  try {
    new URL(params.redirect_uri)
  } catch (e) {
    return 'Invalid redirect_uri format'
  }

  return null
}

// Extract parameters from URL
function extractParams() {
  authParams.value = {
    response_type: route.query.response_type || '',
    client_id: route.query.client_id || '',
    redirect_uri: route.query.redirect_uri || '',
    scope: route.query.scope || '',
    state: route.query.state || '',
    // TODO: PKCE not yet implemented in backend
    code_challenge: route.query.code_challenge || '',
    code_challenge_method: route.query.code_challenge_method || ''
  }

  validationError.value = validateParams()
}

// Handle Authorize click
async function handleAuthorize() {
  try {
    loading.value = true

    // Call backend authorization endpoint with query parameters
    // Note: Backend expects query parameters on POST endpoint (see oauth.rs:57)
    const response = await apiClient.post('/oauth/authorize', null, {
      params: {
        client_id: authParams.value.client_id,
        redirect_uri: authParams.value.redirect_uri,
        response_type: authParams.value.response_type,
        scope: authParams.value.scope,
        state: authParams.value.state
        // Note: code_challenge and code_challenge_method intentionally not sent
        // TODO: Add PKCE support when backend implements it
      }
    })

    // Extract authorization code and state from response
    const { code, state } = response.data

    // Build redirect URL with authorization code
    const redirectUrl = new URL(authParams.value.redirect_uri)
    redirectUrl.searchParams.set('code', code)
    if (state) {
      redirectUrl.searchParams.set('state', state)
    }

    // Redirect to client application
    window.location.href = redirectUrl.toString()
  } catch (error) {
    loading.value = false

    // Handle API errors
    if (error.response) {
      const status = error.response.status
      const errorMessage = error.response.data?.message || error.response.data?.error

      if (status === 401) {
        message.error(errorMessage || 'Invalid client_id')
      } else if (status === 400) {
        message.error(errorMessage || 'Invalid request parameters')
      } else if (status >= 500) {
        message.error('Server error. Please try again later.')
      } else {
        message.error(errorMessage || 'Authorization failed')
      }
    } else if (error.request) {
      message.error('Unable to connect to server. Please check your connection.')
    } else {
      message.error('An unexpected error occurred')
    }

    console.error('Authorization error:', error)
  }
}

// Handle Deny click
function handleDeny() {
  // Build redirect URL with error
  const redirectUrl = new URL(authParams.value.redirect_uri)
  redirectUrl.searchParams.set('error', 'access_denied')
  if (authParams.value.state) {
    redirectUrl.searchParams.set('state', authParams.value.state)
  }

  // Redirect to client application
  window.location.href = redirectUrl.toString()
}

// Initialize on mount
onMounted(() => {
  extractParams()
})
</script>
