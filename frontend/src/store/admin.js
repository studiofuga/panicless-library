import { defineStore } from 'pinia'
import { ref } from 'vue'
import apiClient from '@/api/client'

export const useAdminStore = defineStore('admin', () => {
  const users = ref([])
  const loading = ref(false)
  const error = ref(null)

  async function fetchUsers() {
    loading.value = true
    error.value = null
    try {
      const response = await apiClient.get('/api/admin/users')
      users.value = response.data
      return response.data
    } catch (err) {
      error.value = err.response?.data?.message || 'Failed to fetch users'
      throw err
    } finally {
      loading.value = false
    }
  }

  async function createUser(data) {
    loading.value = true
    error.value = null
    try {
      const response = await apiClient.post('/api/admin/users', data)
      users.value.unshift(response.data)
      return response.data
    } catch (err) {
      error.value = err.response?.data?.message || 'Failed to create user'
      throw err
    } finally {
      loading.value = false
    }
  }

  async function updateUser(id, data) {
    loading.value = true
    error.value = null
    try {
      const response = await apiClient.put(`/api/admin/users/${id}`, data)
      const index = users.value.findIndex(u => u.id === id)
      if (index !== -1) users.value[index] = response.data
      return response.data
    } catch (err) {
      error.value = err.response?.data?.message || 'Failed to update user'
      throw err
    } finally {
      loading.value = false
    }
  }

  async function deleteUser(id) {
    loading.value = true
    error.value = null
    try {
      await apiClient.delete(`/api/admin/users/${id}`)
      users.value = users.value.filter(u => u.id !== id)
    } catch (err) {
      error.value = err.response?.data?.message || 'Failed to delete user'
      throw err
    } finally {
      loading.value = false
    }
  }

  async function resendInvitation(id) {
    loading.value = true
    error.value = null
    try {
      const response = await apiClient.post(`/api/admin/users/${id}/resend-invitation`)
      return response.data
    } catch (err) {
      error.value = err.response?.data?.message || 'Failed to resend invitation'
      throw err
    } finally {
      loading.value = false
    }
  }

  return {
    users,
    loading,
    error,
    fetchUsers,
    createUser,
    updateUser,
    deleteUser,
    resendInvitation
  }
})
