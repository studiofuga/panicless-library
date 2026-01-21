import axios from 'axios'
import { loadConfig, getApiBaseURL } from './config'

// Create axios instance with runtime config
// baseURL will be set after config is loaded
const apiClient = axios.create({
  headers: {
    'Content-Type': 'application/json',
  },
})

// Initialize API client with runtime config
export async function initializeApiClient() {
  const config = await loadConfig()
  apiClient.defaults.baseURL = config.apiBaseURL
  return apiClient
}

// Request interceptor to add JWT token
apiClient.interceptors.request.use(
  (config) => {
    // Import useAuthStore inside the interceptor to avoid timing issues with Pinia
    // This ensures Pinia is initialized before we try to use the store
    const { useAuthStore } = require('@/store/auth')
    const authStore = useAuthStore()
    if (authStore.accessToken) {
      config.headers.Authorization = `Bearer ${authStore.accessToken}`
    }
    return config
  },
  (error) => Promise.reject(error)
)

// Response interceptor to handle token refresh
apiClient.interceptors.response.use(
  (response) => response,
  async (error) => {
    const originalRequest = error.config

    // If error is 401 and we haven't tried to refresh yet
    if (error.response?.status === 401 && !originalRequest._retry) {
      originalRequest._retry = true

      // Import useAuthStore inside the interceptor to avoid timing issues with Pinia
      const { useAuthStore } = require('@/store/auth')
      const authStore = useAuthStore()

      try {
        // Try to refresh the token
        await authStore.refreshToken()

        // Retry the original request with new token
        originalRequest.headers.Authorization = `Bearer ${authStore.accessToken}`
        return apiClient(originalRequest)
      } catch (refreshError) {
        // Refresh failed, logout user
        authStore.logout()
        window.location.href = '/login'
        return Promise.reject(refreshError)
      }
    }

    return Promise.reject(error)
  }
)

export default apiClient
