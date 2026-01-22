import { defineStore } from 'pinia'
import { ref } from 'vue'
import apiClient from '@/api/client'

export const useReadingsStore = defineStore('readings', () => {
  const readings = ref([])
  const stats = ref(null)
  const loading = ref(false)
  const error = ref(null)
  const currentPage = ref(1)
  const pageSize = ref(20)
  const totalReadings = ref(0)

  async function fetchReadings(params = {}) {
    loading.value = true
    error.value = null
    try {
      // Include pagination parameters
      const requestParams = {
        page: currentPage.value,
        limit: pageSize.value,
        ...params
      }
      const response = await apiClient.get('/api/readings', { params: requestParams })
      readings.value = response.data

      // Calculate total if we have less items than pageSize, it's the last page
      if (response.data.length < pageSize.value) {
        totalReadings.value = (currentPage.value - 1) * pageSize.value + response.data.length
      }

      return response.data
    } catch (err) {
      error.value = err.response?.data?.message || 'Failed to fetch readings'
      throw err
    } finally {
      loading.value = false
    }
  }

  function setCurrentPage(page) {
    currentPage.value = page
  }

  function setPageSize(size) {
    pageSize.value = size
    currentPage.value = 1 // Reset to first page when changing page size
  }

  async function createReading(readingData) {
    loading.value = true
    error.value = null
    try {
      const response = await apiClient.post('/api/readings', readingData)
      readings.value.unshift(response.data)
      return response.data
    } catch (err) {
      error.value = err.response?.data?.message || 'Failed to create reading'
      throw err
    } finally {
      loading.value = false
    }
  }

  async function updateReading(id, readingData) {
    loading.value = true
    error.value = null
    try {
      const response = await apiClient.put(`/api/readings/${id}`, readingData)
      const index = readings.value.findIndex(r => r.id === id)
      if (index !== -1) readings.value[index] = response.data
      return response.data
    } catch (err) {
      error.value = err.response?.data?.message || 'Failed to update reading'
      throw err
    } finally {
      loading.value = false
    }
  }

  async function completeReading(id, data) {
    loading.value = true
    error.value = null
    try {
      const response = await apiClient.patch(`/api/readings/${id}/complete`, data)
      const index = readings.value.findIndex(r => r.id === id)
      if (index !== -1) readings.value[index] = response.data
      return response.data
    } catch (err) {
      error.value = err.response?.data?.message || 'Failed to complete reading'
      throw err
    } finally {
      loading.value = false
    }
  }

  async function deleteReading(id) {
    loading.value = true
    error.value = null
    try {
      await apiClient.delete(`/api/readings/${id}`)
      readings.value = readings.value.filter(r => r.id !== id)
    } catch (err) {
      error.value = err.response?.data?.message || 'Failed to delete reading'
      throw err
    } finally {
      loading.value = false
    }
  }

  async function fetchStats() {
    loading.value = true
    error.value = null
    try {
      const response = await apiClient.get('/api/readings/stats')
      stats.value = response.data
      return response.data
    } catch (err) {
      error.value = err.response?.data?.message || 'Failed to fetch statistics'
      throw err
    } finally {
      loading.value = false
    }
  }

  return {
    readings,
    stats,
    loading,
    error,
    currentPage,
    pageSize,
    totalReadings,
    fetchReadings,
    createReading,
    updateReading,
    completeReading,
    deleteReading,
    fetchStats,
    setCurrentPage,
    setPageSize
  }
})
