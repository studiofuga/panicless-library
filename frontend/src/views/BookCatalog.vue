<template>
  <div>
    <n-space justify="space-between" style="margin-bottom: 24px;">
      <h1>My Books</h1>
      <n-button type="primary" @click="showAddModal = true">
        Add Book
      </n-button>
    </n-space>

    <!-- Filters -->
    <n-card size="small" style="margin-bottom: 16px;">
      <n-space align="center" :wrap="true">
        <n-input
          v-model:value="filters.search"
          placeholder="Search keyword..."
          clearable
          style="width: 200px;"
          @update:value="handleFilterChange"
        />
        <n-input
          v-model:value="filters.author"
          placeholder="Author..."
          clearable
          style="width: 180px;"
          @update:value="handleFilterChange"
        />
        <n-input-number
          v-model:value="filters.publication_year"
          placeholder="Year"
          clearable
          :min="1000"
          :max="9999"
          style="width: 140px;"
          @update:value="handleFilterChange"
        />
        <n-popover trigger="click">
          <template #trigger>
            <n-button quaternary>Columns</n-button>
          </template>
          <n-checkbox-group v-model:value="visibleColumns">
            <n-space vertical>
              <n-checkbox value="title" label="Title" disabled />
              <n-checkbox value="author" label="Author" />
              <n-checkbox value="publication_year" label="Year" />
              <n-checkbox value="pages" label="Pages" />
              <n-checkbox value="publisher" label="Publisher" />
              <n-checkbox value="isbn" label="ISBN" />
              <n-checkbox value="language" label="Language" />
            </n-space>
          </n-checkbox-group>
        </n-popover>
        <n-button @click="clearFilters" quaternary>Clear filters</n-button>
      </n-space>
    </n-card>

    <!-- Data Table -->
    <n-spin :show="loading">
      <n-empty v-if="books.length === 0 && !loading" description="No books found. Add your first book!" />
      <n-data-table
        v-else
        :columns="tableColumns"
        :data="books"
        :row-key="row => row.id"
        :bordered="true"
        :row-props="getRowProps"
        :single-line="false"
        remote
        @update:sorter="handleSorterChange"
      />
    </n-spin>

    <!-- Pagination -->
    <n-space justify="center" style="margin-top: 16px;">
      <n-pagination
        :page="booksStore.currentPage"
        :page-size="booksStore.pageSize"
        :item-count="totalItems"
        :page-sizes="[20, 50, 100]"
        show-size-picker
        @update:page="handlePageChange"
        @update:page-size="handlePageSizeChange"
      />
    </n-space>

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
import { useRouter } from 'vue-router'
import { useBooksStore } from '@/store/books'
import { useMessage } from 'naive-ui'
import {
  NSpace,
  NButton,
  NInput,
  NInputNumber,
  NSpin,
  NEmpty,
  NModal,
  NCard,
  NForm,
  NFormItem,
  NGrid,
  NGridItem,
  NPagination,
  NDataTable,
  NPopover,
  NCheckboxGroup,
  NCheckbox
} from 'naive-ui'

const router = useRouter()
const booksStore = useBooksStore()
const message = useMessage()

const books = computed(() => booksStore.books)
const loading = computed(() => booksStore.loading)
const totalItems = computed(() => {
  if (books.value.length < booksStore.pageSize) {
    return (booksStore.currentPage - 1) * booksStore.pageSize + books.value.length
  }
  return (booksStore.currentPage + 1) * booksStore.pageSize
})

const showAddModal = ref(false)
const saving = ref(false)
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

// Filters
const filters = ref({
  search: '',
  author: '',
  publication_year: null
})
let debounceTimer = null

const visibleColumns = ref(['title', 'author', 'publication_year', 'pages'])

// Sorting state
const bookSortBy = ref('title')
const bookSortOrder = ref('ascend')

const sortableKeys = ['title', 'author', 'publication_year', 'pages', 'publisher', 'language']

const allColumns = computed(() => [
  { title: 'Title', key: 'title', resizable: true, minWidth: 150 },
  { title: 'Author', key: 'author', resizable: true, minWidth: 120 },
  { title: 'Year', key: 'publication_year', width: 80 },
  { title: 'Pages', key: 'pages', width: 80 },
  { title: 'Publisher', key: 'publisher', resizable: true, minWidth: 120 },
  { title: 'ISBN', key: 'isbn', width: 150 },
  { title: 'Language', key: 'language', width: 100 }
].map(col => {
  if (sortableKeys.includes(col.key)) {
    return {
      ...col,
      sorter: true,
      sortOrder: bookSortBy.value === col.key ? bookSortOrder.value : false
    }
  }
  return col
}))

const tableColumns = computed(() => {
  return allColumns.value.filter(col => visibleColumns.value.includes(col.key))
})

const getRowProps = (row) => {
  return {
    style: 'cursor: pointer;',
    onClick: () => {
      router.push(`/books/${row.id}`)
    }
  }
}

const handleFilterChange = () => {
  clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => {
    performSearch()
  }, 400)
}

const performSearch = async () => {
  try {
    booksStore.setCurrentPage(1)
    const params = {}
    if (filters.value.search) params.title = filters.value.search
    if (filters.value.author) params.author = filters.value.author
    if (filters.value.publication_year) params.publication_year = filters.value.publication_year

    const sortParams = buildSortParams()
    const hasFilters = Object.keys(params).length > 0
    if (hasFilters) {
      await booksStore.advancedSearch({ ...params, ...sortParams })
    } else {
      await booksStore.fetchBooks(sortParams)
    }
  } catch (error) {
    message.error('Search failed')
  }
}

const clearFilters = () => {
  filters.value = { search: '', author: '', publication_year: null }
  booksStore.setCurrentPage(1)
  booksStore.fetchBooks(buildSortParams())
}

onMounted(async () => {
  try {
    await booksStore.fetchBooks(buildSortParams())
  } catch (error) {
    message.error('Failed to load books')
  }
})

const handlePageChange = async (page) => {
  try {
    booksStore.setCurrentPage(page)
    const filterParams = buildFilterParams()
    const sortParams = buildSortParams()
    if (Object.keys(filterParams).length > 0) {
      await booksStore.advancedSearch({ ...filterParams, ...sortParams })
    } else {
      await booksStore.fetchBooks(sortParams)
    }
  } catch (error) {
    message.error('Failed to load page')
  }
}

const handlePageSizeChange = async (pageSize) => {
  try {
    booksStore.setPageSize(pageSize)
    const filterParams = buildFilterParams()
    const sortParams = buildSortParams()
    if (Object.keys(filterParams).length > 0) {
      await booksStore.advancedSearch({ ...filterParams, ...sortParams })
    } else {
      await booksStore.fetchBooks(sortParams)
    }
  } catch (error) {
    message.error('Failed to change page size')
  }
}

const sortOrderToBackend = (order) => {
  if (order === 'ascend') return 'asc'
  if (order === 'descend') return 'desc'
  return undefined
}

const buildSortParams = () => {
  const params = {}
  if (bookSortBy.value) params.sort_by = bookSortBy.value
  const dir = sortOrderToBackend(bookSortOrder.value)
  if (dir) params.sort_order = dir
  return params
}

const buildFilterParams = () => {
  const params = {}
  if (filters.value.search) params.title = filters.value.search
  if (filters.value.author) params.author = filters.value.author
  if (filters.value.publication_year) params.publication_year = filters.value.publication_year
  return params
}

const handleSorterChange = async (sorter) => {
  if (sorter) {
    bookSortBy.value = sorter.columnKey
    bookSortOrder.value = sorter.order || 'ascend'
  } else {
    bookSortBy.value = 'title'
    bookSortOrder.value = 'ascend'
  }
  try {
    booksStore.setCurrentPage(1)
    const filterParams = buildFilterParams()
    const sortParams = buildSortParams()
    const params = { ...filterParams, ...sortParams }
    if (Object.keys(filterParams).length > 0) {
      await booksStore.advancedSearch(params)
    } else {
      await booksStore.fetchBooks(sortParams)
    }
  } catch (error) {
    message.error('Failed to sort books')
  }
}

const handleAddBook = async () => {
  try {
    saving.value = true
    const data = { ...formValue.value }
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

    filters.value = { search: '', author: '', publication_year: null }
    booksStore.setCurrentPage(1)
    await booksStore.fetchBooks(buildSortParams())
  } catch (error) {
    message.error('Failed to add book')
  } finally {
    saving.value = false
  }
}
</script>
