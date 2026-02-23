<template>
  <div>
    <n-space justify="space-between" style="margin-bottom: 24px;">
      <h1>Reading Tracker</h1>
    </n-space>

    <n-tabs v-model:value="activeTab" type="line" @update:value="handleTabChange">
      <!-- Tab: In lettura -->
      <n-tab-pane name="current" tab="In lettura">
        <n-space style="margin-bottom: 16px;" align="center">
          <n-select
            v-model:value="currentYearFilter"
            :options="yearOptions"
            clearable
            placeholder="Filter by year"
            style="width: 160px"
            @update:value="handleCurrentFilterChange"
          />
        </n-space>

        <n-spin :show="readingsLoading">
          <n-empty v-if="currentReadings.length === 0 && !readingsLoading" description="No current readings." />
          <n-data-table
            v-else
            :columns="currentColumns"
            :data="currentReadings"
            :row-key="row => row.id"
            :bordered="true"
            :row-props="getReadingRowProps"
            :single-line="false"
          />
        </n-spin>

        <n-space justify="center" style="margin-top: 16px;">
          <n-pagination
            :page="readingsStore.currentPage"
            :page-size="readingsStore.pageSize"
            :item-count="currentTotalItems"
            :page-sizes="[20, 50, 100]"
            show-size-picker
            @update:page="handleCurrentPageChange"
            @update:page-size="handleCurrentPageSizeChange"
          />
        </n-space>
      </n-tab-pane>

      <!-- Tab: Completate -->
      <n-tab-pane name="completed" tab="Completate">
        <n-space style="margin-bottom: 16px;" align="center">
          <n-select
            v-model:value="completedYearFilter"
            :options="yearOptions"
            clearable
            placeholder="Filter by year"
            style="width: 160px"
            @update:value="handleCompletedFilterChange"
          />
        </n-space>

        <n-spin :show="readingsLoading">
          <n-empty v-if="completedReadings.length === 0 && !readingsLoading" description="No completed readings." />
          <n-data-table
            v-else
            :columns="completedColumns"
            :data="completedReadings"
            :row-key="row => row.id"
            :bordered="true"
            :row-props="getReadingRowProps"
            :single-line="false"
          />
        </n-spin>

        <n-space justify="center" style="margin-top: 16px;">
          <n-pagination
            :page="readingsStore.currentPage"
            :page-size="readingsStore.pageSize"
            :item-count="completedTotalItems"
            :page-sizes="[20, 50, 100]"
            show-size-picker
            @update:page="handleCompletedPageChange"
            @update:page-size="handleCompletedPageSizeChange"
          />
        </n-space>
      </n-tab-pane>

      <!-- Tab: Non letti -->
      <n-tab-pane name="unread" tab="Non letti">
        <n-spin :show="booksLoading">
          <n-empty v-if="unreadBooks.length === 0 && !booksLoading" description="No unread books." />
          <n-data-table
            v-else
            :columns="unreadColumns"
            :data="unreadBooks"
            :row-key="row => row.id"
            :bordered="true"
            :row-props="getUnreadRowProps"
            :single-line="false"
          />
        </n-spin>

        <n-space justify="center" style="margin-top: 16px;">
          <n-pagination
            :page="booksStore.unreadCurrentPage"
            :page-size="booksStore.unreadPageSize"
            :item-count="unreadTotalItems"
            :page-sizes="[20, 50, 100]"
            show-size-picker
            @update:page="handleUnreadPageChange"
            @update:page-size="handleUnreadPageSizeChange"
          />
        </n-space>
      </n-tab-pane>
    </n-tabs>

    <!-- Complete Reading Modal -->
    <n-modal v-model:show="showCompleteModal">
      <n-card
        style="width: 400px"
        title="Complete Reading"
        :bordered="false"
        size="huge"
      >
        <n-form>
          <n-form-item label="End Date">
            <n-date-picker
              v-model:value="completeData.end_date"
              type="date"
              style="width: 100%"
            />
          </n-form-item>
          <n-form-item label="Rating">
            <n-rate v-model:value="completeData.rating" />
          </n-form-item>
        </n-form>
        <template #footer>
          <n-space justify="end">
            <n-button @click="showCompleteModal = false">Cancel</n-button>
            <n-button type="primary" @click="handleSaveComplete">
              Complete
            </n-button>
          </n-space>
        </template>
      </n-card>
    </n-modal>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, h } from 'vue'
import { useRouter } from 'vue-router'
import { useReadingsStore } from '@/store/readings'
import { useBooksStore } from '@/store/books'
import { useMessage, useDialog } from 'naive-ui'
import { format as formatDateFn } from 'date-fns'
import {
  NSpace,
  NSelect,
  NSpin,
  NEmpty,
  NButton,
  NRate,
  NModal,
  NCard,
  NForm,
  NFormItem,
  NDatePicker,
  NPagination,
  NDataTable,
  NTabs,
  NTabPane
} from 'naive-ui'

const router = useRouter()
const readingsStore = useReadingsStore()
const booksStore = useBooksStore()
const message = useMessage()
const dialog = useDialog()

const activeTab = ref('current')

// Separate data for each readings tab to avoid conflicts
const currentReadings = ref([])
const completedReadings = ref([])
const readingsLoading = computed(() => readingsStore.loading)
const booksLoading = computed(() => booksStore.loading)
const unreadBooks = computed(() => booksStore.unreadBooks)

// Year filter
const currentYearFilter = ref(null)
const completedYearFilter = ref(null)
const currentYear = new Date().getFullYear()
const yearOptions = Array.from({ length: 21 }, (_, i) => {
  const y = currentYear - i
  return { label: String(y), value: y }
})

// Total items for pagination
const currentTotalItems = ref(0)
const completedTotalItems = ref(0)
const unreadTotalItems = computed(() => booksStore.totalUnreadBooks)

// Complete modal state
const showCompleteModal = ref(false)
const selectedReading = ref(null)
const completeData = ref({
  end_date: Date.now(),
  rating: 5
})

const formatDate = (date) => {
  if (!date) return ''
  return formatDateFn(new Date(date), 'MMM dd, yyyy')
}

// --- Column definitions ---

const currentColumns = [
  { title: 'Title', key: 'book_title', resizable: true, minWidth: 150 },
  { title: 'Author', key: 'book_author', resizable: true, minWidth: 120 },
  {
    title: 'Start Date',
    key: 'start_date',
    width: 140,
    defaultSortOrder: 'descend',
    sorter: (a, b) => new Date(a.start_date) - new Date(b.start_date),
    render: (row) => formatDate(row.start_date)
  },
  {
    title: 'Notes',
    key: 'notes',
    resizable: true,
    minWidth: 100,
    ellipsis: { tooltip: true }
  },
  {
    title: 'Actions',
    key: 'actions',
    width: 220,
    render: (row) => {
      return h(NSpace, {}, {
        default: () => [
          h(NButton, {
            size: 'small',
            type: 'primary',
            onClick: (e) => { e.stopPropagation(); handleCompleteReading(row) }
          }, { default: () => 'Mark as Completed' }),
          h(NButton, {
            size: 'small',
            type: 'error',
            ghost: true,
            onClick: (e) => { e.stopPropagation(); handleDeleteReading(row.id) }
          }, { default: () => 'Delete' })
        ]
      })
    }
  }
]

const completedColumns = [
  { title: 'Title', key: 'book_title', resizable: true, minWidth: 150 },
  { title: 'Author', key: 'book_author', resizable: true, minWidth: 120 },
  {
    title: 'Start Date',
    key: 'start_date',
    width: 140,
    render: (row) => formatDate(row.start_date)
  },
  {
    title: 'End Date',
    key: 'end_date',
    width: 140,
    defaultSortOrder: 'descend',
    sorter: (a, b) => new Date(a.end_date) - new Date(b.end_date),
    render: (row) => formatDate(row.end_date)
  },
  {
    title: 'Rating',
    key: 'rating',
    width: 140,
    render: (row) => {
      if (!row.rating) return ''
      return h(NRate, { value: row.rating, readonly: true, size: 'small' })
    }
  },
  {
    title: 'Notes',
    key: 'notes',
    resizable: true,
    minWidth: 100,
    ellipsis: { tooltip: true }
  },
  {
    title: 'Actions',
    key: 'actions',
    width: 80,
    render: (row) => {
      return h(NButton, {
        size: 'small',
        type: 'error',
        ghost: true,
        onClick: (e) => { e.stopPropagation(); handleDeleteReading(row.id) }
      }, { default: () => 'Delete' })
    }
  }
]

const unreadColumns = [
  { title: 'Title', key: 'title', resizable: true, minWidth: 150 },
  { title: 'Author', key: 'author', resizable: true, minWidth: 120 },
  { title: 'Year', key: 'publication_year', width: 80 },
  { title: 'Pages', key: 'pages', width: 80 }
]

// --- Row props (clickable rows) ---

const getReadingRowProps = (row) => ({
  style: 'cursor: pointer;',
  onClick: () => router.push(`/books/${row.book_id}`)
})

const getUnreadRowProps = (row) => ({
  style: 'cursor: pointer;',
  onClick: () => router.push(`/books/${row.id}`)
})

// --- Data fetching ---

const fetchCurrentReadings = async () => {
  const params = { status: 'current' }
  if (currentYearFilter.value) params.year = currentYearFilter.value
  const data = await readingsStore.fetchReadings(params)
  currentReadings.value = data
  if (data.length < readingsStore.pageSize) {
    currentTotalItems.value = (readingsStore.currentPage - 1) * readingsStore.pageSize + data.length
  } else {
    currentTotalItems.value = (readingsStore.currentPage + 1) * readingsStore.pageSize
  }
}

const fetchCompletedReadings = async () => {
  const params = { status: 'completed' }
  if (completedYearFilter.value) params.year = completedYearFilter.value
  const data = await readingsStore.fetchReadings(params)
  completedReadings.value = data
  if (data.length < readingsStore.pageSize) {
    completedTotalItems.value = (readingsStore.currentPage - 1) * readingsStore.pageSize + data.length
  } else {
    completedTotalItems.value = (readingsStore.currentPage + 1) * readingsStore.pageSize
  }
}

// --- Tab change ---

const handleTabChange = async (tab) => {
  try {
    readingsStore.setCurrentPage(1)
    if (tab === 'current') {
      await fetchCurrentReadings()
    } else if (tab === 'completed') {
      await fetchCompletedReadings()
    } else if (tab === 'unread') {
      booksStore.setUnreadCurrentPage(1)
      await booksStore.fetchUnreadBooks()
    }
  } catch (error) {
    message.error('Failed to load data')
  }
}

// --- Filter changes ---

const handleCurrentFilterChange = async () => {
  try {
    readingsStore.setCurrentPage(1)
    await fetchCurrentReadings()
  } catch (error) {
    message.error('Failed to filter readings')
  }
}

const handleCompletedFilterChange = async () => {
  try {
    readingsStore.setCurrentPage(1)
    await fetchCompletedReadings()
  } catch (error) {
    message.error('Failed to filter readings')
  }
}

// --- Pagination: Current ---

const handleCurrentPageChange = async (page) => {
  try {
    readingsStore.setCurrentPage(page)
    await fetchCurrentReadings()
  } catch (error) {
    message.error('Failed to load page')
  }
}

const handleCurrentPageSizeChange = async (size) => {
  try {
    readingsStore.setPageSize(size)
    await fetchCurrentReadings()
  } catch (error) {
    message.error('Failed to change page size')
  }
}

// --- Pagination: Completed ---

const handleCompletedPageChange = async (page) => {
  try {
    readingsStore.setCurrentPage(page)
    await fetchCompletedReadings()
  } catch (error) {
    message.error('Failed to load page')
  }
}

const handleCompletedPageSizeChange = async (size) => {
  try {
    readingsStore.setPageSize(size)
    await fetchCompletedReadings()
  } catch (error) {
    message.error('Failed to change page size')
  }
}

// --- Pagination: Unread ---

const handleUnreadPageChange = async (page) => {
  try {
    booksStore.setUnreadCurrentPage(page)
    await booksStore.fetchUnreadBooks()
  } catch (error) {
    message.error('Failed to load page')
  }
}

const handleUnreadPageSizeChange = async (size) => {
  try {
    booksStore.setUnreadPageSize(size)
    await booksStore.fetchUnreadBooks()
  } catch (error) {
    message.error('Failed to change page size')
  }
}

// --- Actions ---

const handleCompleteReading = (reading) => {
  selectedReading.value = reading
  completeData.value = {
    end_date: Date.now(),
    rating: 5
  }
  showCompleteModal.value = true
}

const handleSaveComplete = async () => {
  try {
    const formattedDate = formatDateFn(new Date(completeData.value.end_date), 'yyyy-MM-dd')
    await readingsStore.completeReading(selectedReading.value.id, {
      end_date: formattedDate,
      rating: completeData.value.rating
    })
    message.success('Reading completed!')
    showCompleteModal.value = false
    readingsStore.setCurrentPage(1)
    await fetchCurrentReadings()
  } catch (error) {
    message.error('Failed to complete reading')
  }
}

const handleDeleteReading = (id) => {
  dialog.warning({
    title: 'Delete Reading',
    content: 'Are you sure you want to delete this reading record?',
    positiveText: 'Delete',
    negativeText: 'Cancel',
    onPositiveClick: async () => {
      try {
        await readingsStore.deleteReading(id)
        message.success('Reading deleted')
        readingsStore.setCurrentPage(1)
        if (activeTab.value === 'current') {
          await fetchCurrentReadings()
        } else {
          await fetchCompletedReadings()
        }
      } catch (error) {
        message.error('Failed to delete reading')
      }
    }
  })
}

// --- Init ---

onMounted(async () => {
  try {
    await fetchCurrentReadings()
  } catch (error) {
    message.error('Failed to load readings')
  }
})
</script>
