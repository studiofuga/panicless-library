<template>
  <div>
    <n-space justify="space-between" style="margin-bottom: 24px;">
      <h1>Reading Tracker</h1>
    </n-space>

    <!-- Filter -->
    <n-space style="margin-bottom: 24px;">
      <n-select
        v-model:value="statusFilter"
        :options="statusOptions"
        style="width: 200px"
        @update:value="handleFilterChange"
      />
    </n-space>

    <!-- Readings List -->
    <n-spin :show="loading">
      <n-empty v-if="readings.length === 0 && !loading" description="No readings found. Start reading a book!" />
      <n-list v-else bordered>
        <n-list-item v-for="reading in readings" :key="reading.id">
          <n-thing>
            <template #header>
              {{ reading.book_title }}
            </template>
            <template #description>
              <n-space vertical size="small">
                <div>by {{ reading.book_author || 'Unknown' }}</div>
                <div>
                  <strong>Started:</strong> {{ formatDate(reading.start_date) }}
                  <span v-if="reading.end_date">
                    | <strong>Finished:</strong> {{ formatDate(reading.end_date) }}
                  </span>
                  <n-tag v-else type="info" size="small" style="margin-left: 8px;">
                    Currently Reading
                  </n-tag>
                </div>
                <div v-if="reading.rating">
                  <strong>Rating:</strong>
                  <n-rate :value="reading.rating" readonly size="small" />
                </div>
                <div v-if="reading.notes">
                  <strong>Notes:</strong> {{ reading.notes }}
                </div>
              </n-space>
            </template>
            <template #footer>
              <n-space>
                <n-button
                  v-if="!reading.end_date"
                  size="small"
                  type="primary"
                  @click="handleCompleteReading(reading)"
                >
                  Mark as Completed
                </n-button>
                <n-button
                  size="small"
                  type="error"
                  ghost
                  @click="handleDeleteReading(reading.id)"
                >
                  Delete
                </n-button>
              </n-space>
            </template>
          </n-thing>
        </n-list-item>
      </n-list>
    </n-spin>

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
import { ref, computed, onMounted } from 'vue'
import { useReadingsStore } from '@/store/readings'
import { useMessage, useDialog } from 'naive-ui'
import { format as formatDateFn } from 'date-fns'
import {
  NSpace,
  NSelect,
  NSpin,
  NEmpty,
  NList,
  NListItem,
  NThing,
  NTag,
  NButton,
  NRate,
  NModal,
  NCard,
  NForm,
  NFormItem,
  NDatePicker
} from 'naive-ui'

const readingsStore = useReadingsStore()
const message = useMessage()
const dialog = useDialog()

const readings = computed(() => readingsStore.readings)
const loading = computed(() => readingsStore.loading)

const statusFilter = ref('all')
const statusOptions = [
  { label: 'All Readings', value: 'all' },
  { label: 'Currently Reading', value: 'current' },
  { label: 'Completed', value: 'completed' }
]

const showCompleteModal = ref(false)
const selectedReading = ref(null)
const completeData = ref({
  end_date: Date.now(),
  rating: 5
})

onMounted(async () => {
  try {
    await readingsStore.fetchReadings()
  } catch (error) {
    message.error('Failed to load readings')
  }
})

const handleFilterChange = async () => {
  try {
    const params = statusFilter.value === 'all' ? {} : { status: statusFilter.value }
    await readingsStore.fetchReadings(params)
  } catch (error) {
    message.error('Failed to filter readings')
  }
}

const formatDate = (date) => {
  if (!date) return 'N/A'
  return formatDateFn(new Date(date), 'MMM dd, yyyy')
}

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
    await readingsStore.fetchReadings({ status: statusFilter.value === 'all' ? undefined : statusFilter.value })
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
      } catch (error) {
        message.error('Failed to delete reading')
      }
    }
  })
}
</script>
