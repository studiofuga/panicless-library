import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import apiClient from '@/api/client'

export const useConnectorsStore = defineStore('connectors', () => {
  // State
  const connectors = ref([])
  const loading = ref(false)
  const error = ref(null)

  // Computed
  const hasConnector = computed(() => (provider) => {
    return connectors.value.some(c => c.provider === provider && c.is_active)
  })

  const getConnector = computed(() => (provider) => {
    return connectors.value.find(c => c.provider === provider)
  })

  const activeConnectors = computed(() => {
    return connectors.value.filter(c => c.is_active)
  })

  // Actions
  async function fetchConnectors() {
    loading.value = true
    error.value = null
    try {
      const response = await apiClient.get('/api/connectors')
      connectors.value = response.data
      return response.data
    } catch (err) {
      error.value = err.response?.data?.message || 'Failed to fetch connectors'
      throw err
    } finally {
      loading.value = false
    }
  }

  async function createOrUpdateConnector(provider, apiToken) {
    loading.value = true
    error.value = null
    try {
      const response = await apiClient.post('/api/connectors', {
        provider,
        api_token: apiToken
      })

      // Update local state
      const existingIndex = connectors.value.findIndex(c => c.provider === provider)
      if (existingIndex >= 0) {
        connectors.value[existingIndex] = response.data
      } else {
        connectors.value.push(response.data)
      }

      return response.data
    } catch (err) {
      error.value = err.response?.data?.message || 'Failed to create/update connector'
      throw err
    } finally {
      loading.value = false
    }
  }

  async function deleteConnector(provider) {
    loading.value = true
    error.value = null
    try {
      await apiClient.delete(`/api/connectors/${provider}`)

      // Update local state
      connectors.value = connectors.value.filter(c => c.provider !== provider)

      return true
    } catch (err) {
      error.value = err.response?.data?.message || 'Failed to delete connector'
      throw err
    } finally {
      loading.value = false
    }
  }

  async function toggleConnector(provider) {
    loading.value = true
    error.value = null
    try {
      const response = await apiClient.patch(`/api/connectors/${provider}/toggle`)

      // Update local state
      const index = connectors.value.findIndex(c => c.provider === provider)
      if (index >= 0) {
        connectors.value[index] = response.data
      }

      return response.data
    } catch (err) {
      error.value = err.response?.data?.message || 'Failed to toggle connector'
      throw err
    } finally {
      loading.value = false
    }
  }

  return {
    // State
    connectors,
    loading,
    error,

    // Computed
    hasConnector,
    getConnector,
    activeConnectors,

    // Actions
    fetchConnectors,
    createOrUpdateConnector,
    deleteConnector,
    toggleConnector
  }
})
