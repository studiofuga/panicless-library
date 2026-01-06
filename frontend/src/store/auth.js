import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import apiClient from '@/api/client'

export const useAuthStore = defineStore('auth', () => {
  // State
  const user = ref(null)
  const accessToken = ref(localStorage.getItem('access_token') || null)
  const refreshTokenValue = ref(localStorage.getItem('refresh_token') || null)

  // Getters
  const isAuthenticated = computed(() => !!accessToken.value)
  const currentUser = computed(() => user.value)

  // Actions
  async function register(userData) {
    const response = await apiClient.post('/api/auth/register', userData)
    setAuthData(response.data)
    return response.data
  }

  async function login(credentials) {
    const response = await apiClient.post('/api/auth/login', credentials)
    setAuthData(response.data)
    return response.data
  }

  async function refreshToken() {
    if (!refreshTokenValue.value) {
      throw new Error('No refresh token available')
    }

    const response = await apiClient.post('/api/auth/refresh', {
      refresh_token: refreshTokenValue.value
    })
    setAuthData(response.data)
    return response.data
  }

  async function fetchCurrentUser() {
    if (!accessToken.value) return null

    try {
      const response = await apiClient.get('/api/auth/me')
      user.value = response.data
      return response.data
    } catch (error) {
      console.error('Failed to fetch current user:', error)
      logout()
      return null
    }
  }

  function setAuthData(data) {
    accessToken.value = data.access_token
    refreshTokenValue.value = data.refresh_token
    user.value = data.user

    localStorage.setItem('access_token', data.access_token)
    localStorage.setItem('refresh_token', data.refresh_token)
  }

  function logout() {
    user.value = null
    accessToken.value = null
    refreshTokenValue.value = null

    localStorage.removeItem('access_token')
    localStorage.removeItem('refresh_token')
  }

  // Initialize: fetch user if token exists
  if (accessToken.value && !user.value) {
    fetchCurrentUser()
  }

  return {
    user,
    accessToken,
    isAuthenticated,
    currentUser,
    register,
    login,
    refreshToken,
    fetchCurrentUser,
    logout
  }
})
