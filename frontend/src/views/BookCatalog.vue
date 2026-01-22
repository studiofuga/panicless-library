<template>
  <div>
    <n-space justify="space-between" style="margin-bottom: 24px;">
      <h1>My Books</h1>
      <n-button type="primary" @click="showAddModal = true">
        Add Book
      </n-button>
    </n-space>

    <!-- Search -->
    <n-space style="margin-bottom: 24px;">
      <n-input
        v-model:value="searchQuery"
        placeholder="Search books..."
        @update:value="handleSearch"
        clearable
      />
    </n-space>

    <!-- Books List -->
    <n-spin :show="loading">
      <n-empty v-if="books.length === 0 && !loading" description="No books found. Add your first book!" />
      <div v-else>
        <n-list hoverable clickable>
          <n-list-item v-for="book in books" :key="book.id" @click="$router.push(`/books/${book.id}`)">
            <n-thing>
              <template #header>
                {{ book.title }}
              </template>
              <template #description>
                {{ book.author || 'Unknown Author' }}
                <span v-if="book.publication_year"> ({{ book.publication_year }})</span>
              </template>
              <template #footer>
                <n-space size="small">
                  <n-tag v-if="book.isbn" size="small">ISBN: {{ book.isbn }}</n-tag>
                  <n-tag v-if="book.publisher" size="small">{{ book.publisher }}</n-tag>
                </n-space>
              </template>
            </n-thing>
          </n-list-item>
        </n-list>

        <!-- Pagination -->
        <n-space justify="center" style="margin-top: 24px;">
          <n-pagination
            :page="booksStore.currentPage"
            :page-size="booksStore.pageSize"
            :item-count="totalItems"
            :on-update:page="handlePageChange"
            @update:page-size="handlePageSizeChange"
          />
        </n-space>
      </div>
    </n-spin>

    <!-- Add Book Modal -->
    <n-modal v-model:show="showAddModal">
      <n-card
        style="width: 600px"
        title="Add New Book"
        :bordered="false"
        size="huge"
        role="dialog"
        aria-modal="true"
      >
        <n-form ref="formRef" :model="formValue">
          <n-form-item label="Title" path="title">
            <n-input v-model:value="formValue.title" placeholder="Book title" />
          </n-form-item>
          <n-form-item label="Author" path="author">
            <n-input v-model:value="formValue.author" placeholder="Author name" />
          </n-form-item>
          <n-form-item label="ISBN" path="isbn">
            <n-input v-model:value="formValue.isbn" placeholder="ISBN-13" />
          </n-form-item>
          <n-grid cols="2" x-gap="12">
            <n-grid-item>
              <n-form-item label="Publication Year" path="publication_year">
                <n-input-number v-model:value="formValue.publication_year" :min="1000" :max="9999" style="width: 100%" />
              </n-form-item>
            </n-grid-item>
            <n-grid-item>
              <n-form-item label="Pages" path="pages">
                <n-input-number v-model:value="formValue.pages" :min="1" style="width: 100%" />
              </n-form-item>
            </n-grid-item>
          </n-grid>
          <n-form-item label="Publisher" path="publisher">
            <n-input v-model:value="formValue.publisher" placeholder="Publisher name" />
          </n-form-item>
          <n-form-item label="Language" path="language">
            <n-input v-model:value="formValue.language" placeholder="e.g., English" />
          </n-form-item>
          <n-form-item label="Description" path="description">
            <n-input
              v-model:value="formValue.description"
              type="textarea"
              placeholder="Book description"
              :rows="3"
            />
          </n-form-item>
        </n-form>
        <template #footer>
          <n-space justify="end">
            <n-button @click="showAddModal = false">Cancel</n-button>
            <n-button type="primary" :loading="saving" @click="handleAddBook">
              Add Book
            </n-button>
          </n-space>
        </template>
      </n-card>
    </n-modal>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useBooksStore } from '@/store/books'
import { useMessage } from 'naive-ui'
import {
  NSpace,
  NButton,
  NInput,
  NSpin,
  NEmpty,
  NList,
  NListItem,
  NThing,
  NTag,
  NModal,
  NCard,
  NForm,
  NFormItem,
  NInputNumber,
  NGrid,
  NGridItem,
  NPagination
} from 'naive-ui'

const booksStore = useBooksStore()
const message = useMessage()

const books = computed(() => booksStore.books)
const loading = computed(() => booksStore.loading)
const totalItems = computed(() => {
  // If we're on the last page and got less items than pageSize, we can calculate exact total
  // Otherwise, we estimate based on full pages
  if (books.value.length < booksStore.pageSize) {
    return (booksStore.currentPage - 1) * booksStore.pageSize + books.value.length
  }
  // For estimation: assume there's at least one more page
  return (booksStore.currentPage + 1) * booksStore.pageSize
})

const showAddModal = ref(false)
const saving = ref(false)
const searchQuery = ref('')
const currentSearchQuery = ref('')
const formRef = ref(null)
const formValue = ref({
  title: '',
  author: '',
  isbn: '',
  publication_year: null,
  pages: null,
  publisher: '',
  language: '',
  description: ''
})

onMounted(async () => {
  try {
    await booksStore.fetchBooks()
  } catch (error) {
    message.error('Failed to load books')
  }
})

const handleSearch = async () => {
  try {
    currentSearchQuery.value = searchQuery.value
    booksStore.setCurrentPage(1) // Reset to first page on new search
    await booksStore.fetchBooks({ search: searchQuery.value })
  } catch (error) {
    message.error('Search failed')
  }
}

const handlePageChange = async (page) => {
  try {
    booksStore.setCurrentPage(page)
    const params = currentSearchQuery.value ? { search: currentSearchQuery.value } : {}
    await booksStore.fetchBooks(params)
  } catch (error) {
    message.error('Failed to load page')
  }
}

const handlePageSizeChange = async (pageSize) => {
  try {
    booksStore.setPageSize(pageSize)
    const params = currentSearchQuery.value ? { search: currentSearchQuery.value } : {}
    await booksStore.fetchBooks(params)
  } catch (error) {
    message.error('Failed to change page size')
  }
}

const handleAddBook = async () => {
  try {
    saving.value = true
    const data = { ...formValue.value }
    // Remove null/empty values
    Object.keys(data).forEach(key => {
      if (data[key] === null || data[key] === '') delete data[key]
    })

    if (!data.title) {
      message.error('Title is required')
      return
    }

    await booksStore.createBook(data)
    message.success('Book added successfully!')
    showAddModal.value = false

    // Reset form
    formValue.value = {
      title: '',
      author: '',
      isbn: '',
      publication_year: null,
      pages: null,
      publisher: '',
      language: '',
      description: ''
    }

    // Reload books list (reset to first page and clear search)
    searchQuery.value = ''
    currentSearchQuery.value = ''
    booksStore.setCurrentPage(1)
    await booksStore.fetchBooks()
  } catch (error) {
    message.error('Failed to add book')
  } finally {
    saving.value = false
  }
}
</script>
