import axios from 'axios'
import { useAuthStore } from '@/store/auth'
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
    // Read token directly from localStorage to avoid issues with store initialization
    const accessToken = localStorage.getItem('access_token')
    if (accessToken) {
      config.headers.Authorization = `Bearer ${accessToken}`
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

      const authStore = useAuthStore()

      try {
        // Try to refresh the token
        await authStore.refreshToken()

        // Read the new token from localStorage (updated by authStore.refreshToken())
        const newAccessToken = localStorage.getItem('access_token')
        if (newAccessToken) {
          originalRequest.headers.Authorization = `Bearer ${newAccessToken}`
        }
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
