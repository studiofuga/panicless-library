<template>
  <div>
    <n-button text @click="$router.back()" style="margin-bottom: 16px;">
      ← Back to Books
    </n-button>

    <n-spin :show="loading">
      <n-card v-if="book" :title="book.title">
        <n-descriptions bordered :column="2">
          <n-descriptions-item label="Author">
            {{ book.author || 'N/A' }}
          </n-descriptions-item>
          <n-descriptions-item label="Publication Year">
            {{ book.publication_year || 'N/A' }}
          </n-descriptions-item>
          <n-descriptions-item label="ISBN">
            {{ book.isbn || 'N/A' }}
          </n-descriptions-item>
          <n-descriptions-item label="Publisher">
            {{ book.publisher || 'N/A' }}
          </n-descriptions-item>
          <n-descriptions-item label="Pages">
            {{ book.pages || 'N/A' }}
          </n-descriptions-item>
          <n-descriptions-item label="Language">
            {{ book.language || 'N/A' }}
          </n-descriptions-item>
          <n-descriptions-item label="Edition" :span="2">
            {{ book.edition || 'N/A' }}
          </n-descriptions-item>
          <n-descriptions-item label="Description" :span="2">
            {{ book.description || 'No description available' }}
          </n-descriptions-item>
        </n-descriptions>

        <template #footer>
          <n-space>
            <n-button type="primary" @click="showStartReadingModal = true">
              Start Reading
            </n-button>
            <n-button @click="handleDelete" type="error" ghost>
              Delete Book
            </n-button>
          </n-space>
        </template>
      </n-card>
    </n-spin>

    <!-- Start Reading Modal -->
    <n-modal v-model:show="showStartReadingModal">
      <n-card
        style="width: 400px"
        title="Start Reading"
        :bordered="false"
        size="huge"
      >
        <n-form>
          <n-form-item label="Start Date">
            <n-date-picker
              v-model:value="startDate"
              type="date"
              style="width: 100%"
            />
          </n-form-item>
        </n-form>
        <template #footer>
          <n-space justify="end">
            <n-button @click="showStartReadingModal = false">Cancel</n-button>
            <n-button type="primary" @click="handleStartReading">
              Start
            </n-button>
          </n-space>
        </template>
      </n-card>
    </n-modal>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useBooksStore } from '@/store/books'
import { useReadingsStore } from '@/store/readings'
import { useMessage, useDialog } from 'naive-ui'
import { format } from 'date-fns'
import {
  NButton,
  NSpin,
  NCard,
  NDescriptions,
  NDescriptionsItem,
  NSpace,
  NModal,
  NForm,
  NFormItem,
  NDatePicker
} from 'naive-ui'

const route = useRoute()
const router = useRouter()
const booksStore = useBooksStore()
const readingsStore = useReadingsStore()
const message = useMessage()
const dialog = useDialog()

const book = computed(() => booksStore.currentBook)
const loading = computed(() => booksStore.loading)

const showStartReadingModal = ref(false)
const startDate = ref(Date.now())

onMounted(async () => {
  const bookId = parseInt(route.params.id)
  try {
    await booksStore.fetchBook(bookId)
  } catch (error) {
    message.error('Failed to load book')
    router.push('/books')
  }
})

const handleStartReading = async () => {
  try {
    const formattedDate = format(new Date(startDate.value), 'yyyy-MM-dd')
    await readingsStore.createReading({
      book_id: book.value.id,
      start_date: formattedDate,
      end_date: null,
      rating: null,
      notes: null
    })
    message.success('Reading started!')
    showStartReadingModal.value = false
    router.push('/readings')
  } catch (error) {
    message.error(error.response?.data?.message || 'Failed to start reading')
  }
}

const handleDelete = () => {
  dialog.warning({
    title: 'Delete Book',
    content: 'Are you sure you want to delete this book? This action cannot be undone.',
    positiveText: 'Delete',
    negativeText: 'Cancel',
    onPositiveClick: async () => {
      try {
        await booksStore.deleteBook(book.value.id)
        message.success('Book deleted')
        router.push('/books')
      } catch (error) {
        message.error('Failed to delete book')
      }
    }
  })
}
</script>
