import { defineStore } from 'pinia'
import { ref } from 'vue'
import apiClient from '@/api/client'

export const useBooksStore = defineStore('books', () => {
  const books = ref([])
  const unreadBooks = ref([])
  const currentBook = ref(null)
  const loading = ref(false)
  const error = ref(null)
  const currentPage = ref(1)
  const pageSize = ref(20)
  const totalBooks = ref(0)
  const unreadCurrentPage = ref(1)
  const unreadPageSize = ref(20)
  const totalUnreadBooks = ref(0)

  async function fetchBooks(params = {}) {
    loading.value = true
    error.value = null
    try {
      // Include pagination parameters
      const requestParams = {
        page: currentPage.value,
        limit: pageSize.value,
        ...params
      }
      const response = await apiClient.get('/api/books', { params: requestParams })
      books.value = response.data

      // Calculate total if we have less items than pageSize, it's the last page
      if (response.data.length < pageSize.value) {
        totalBooks.value = (currentPage.value - 1) * pageSize.value + response.data.length
      }

      return response.data
    } catch (err) {
      error.value = err.response?.data?.message || 'Failed to fetch books'
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

  async function fetchUnreadBooks(params = {}) {
    loading.value = true
    error.value = null
    try {
      const requestParams = {
        page: unreadCurrentPage.value,
        limit: unreadPageSize.value,
        ...params
      }
      const response = await apiClient.get('/api/books/unread', { params: requestParams })
      unreadBooks.value = response.data

      if (response.data.length < unreadPageSize.value) {
        totalUnreadBooks.value = (unreadCurrentPage.value - 1) * unreadPageSize.value + response.data.length
      } else {
        totalUnreadBooks.value = (unreadCurrentPage.value + 1) * unreadPageSize.value
      }

      return response.data
    } catch (err) {
      error.value = err.response?.data?.message || 'Failed to fetch unread books'
      throw err
    } finally {
      loading.value = false
    }
  }

  function setUnreadCurrentPage(page) {
    unreadCurrentPage.value = page
  }

  function setUnreadPageSize(size) {
    unreadPageSize.value = size
    unreadCurrentPage.value = 1
  }

  async function advancedSearch(filters = {}) {
    loading.value = true
    error.value = null
    try {
      // Include pagination parameters
      const requestParams = {
        page: currentPage.value,
        limit: pageSize.value,
        ...filters
      }
      const response = await apiClient.get('/api/books/search/advanced', { params: requestParams })
      books.value = response.data

      // Calculate total if we have less items than pageSize, it's the last page
      if (response.data.length < pageSize.value) {
        totalBooks.value = (currentPage.value - 1) * pageSize.value + response.data.length
      }

      return response.data
    } catch (err) {
      error.value = err.response?.data?.message || 'Advanced search failed'
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
    unreadBooks,
    currentBook,
    loading,
    error,
    currentPage,
    pageSize,
    totalBooks,
    unreadCurrentPage,
    unreadPageSize,
    totalUnreadBooks,
    fetchBooks,
    fetchUnreadBooks,
    fetchBook,
    createBook,
    updateBook,
    deleteBook,
    importGoodreadsCSV,
    setCurrentPage,
    setPageSize,
    setUnreadCurrentPage,
    setUnreadPageSize,
    advancedSearch
  }
})
