import { defineStore } from 'pinia'
import { ref } from 'vue'
import apiClient from '@/api/client'

export const useBooksStore = defineStore('books', () => {
  const books = ref([])
  const currentBook = ref(null)
  const loading = ref(false)
  const error = ref(null)

  async function fetchBooks(params = {}) {
    loading.value = true
    error.value = null
    try {
      const response = await apiClient.get('/api/books', { params })
      books.value = response.data
      return response.data
    } catch (err) {
      error.value = err.response?.data?.message || 'Failed to fetch books'
      throw err
    } finally {
      loading.value = false
    }
  }

  async function fetchBook(id) {
    loading.value = true
    error.value = null
    try {
      const response = await apiClient.get(`/api/books/${id}`)
      currentBook.value = response.data
      return response.data
    } catch (err) {
      error.value = err.response?.data?.message || 'Failed to fetch book'
      throw err
    } finally {
      loading.value = false
    }
  }

  async function createBook(bookData) {
    loading.value = true
    error.value = null
    try {
      const response = await apiClient.post('/api/books', bookData)
      books.value.unshift(response.data)
      return response.data
    } catch (err) {
      error.value = err.response?.data?.message || 'Failed to create book'
      throw err
    } finally {
      loading.value = false
    }
  }

  async function updateBook(id, bookData) {
    loading.value = true
    error.value = null
    try {
      const response = await apiClient.put(`/api/books/${id}`, bookData)
      const index = books.value.findIndex(b => b.id === id)
      if (index !== -1) books.value[index] = response.data
      currentBook.value = response.data
      return response.data
    } catch (err) {
      error.value = err.response?.data?.message || 'Failed to update book'
      throw err
    } finally {
      loading.value = false
    }
  }

  async function deleteBook(id) {
    loading.value = true
    error.value = null
    try {
      await apiClient.delete(`/api/books/${id}`)
      books.value = books.value.filter(b => b.id !== id)
    } catch (err) {
      error.value = err.response?.data?.message || 'Failed to delete book'
      throw err
    } finally {
      loading.value = false
    }
  }

  async function importGoodreadsCSV(file) {
    loading.value = true
    error.value = null
    try {
      const formData = new FormData()
      formData.append('file', file)

      const response = await apiClient.post('/api/import/goodreads/csv', formData, {
        headers: {
          'Content-Type': 'multipart/form-data'
        }
      })

      return response.data
    } catch (err) {
      error.value = err.response?.data?.message || 'Failed to import CSV'
      throw err
    } finally {
      loading.value = false
    }
  }

  return {
    books,
    currentBook,
    loading,
    error,
    fetchBooks,
    fetchBook,
    createBook,
    updateBook,
    deleteBook,
    importGoodreadsCSV
  }
})
