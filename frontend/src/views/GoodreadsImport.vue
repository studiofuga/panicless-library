<template>
  <div>
    <n-space vertical size="large">
      <div>
        <h1>Import from Goodreads</h1>
        <p style="color: #666; margin-top: 8px;">
          Upload your Goodreads CSV export to import your books and reading history.
        </p>
      </div>

      <!-- Upload Section -->
      <n-card title="Upload CSV File" :bordered="true">
        <n-space vertical size="large">
          <n-alert type="info" title="How to export from Goodreads">
            <ol style="margin: 8px 0 0 0; padding-left: 20px;">
              <li>Go to Goodreads and navigate to "My Books"</li>
              <li>Click "Import and export" at the top</li>
              <li>Click "Export Library" to download your CSV file</li>
              <li>Upload the downloaded CSV file below</li>
            </ol>
          </n-alert>

          <n-upload
            :custom-request="handleUpload"
            :show-file-list="true"
            :max="1"
            accept=".csv"
            @change="handleFileChange"
            :disabled="importing"
          >
            <n-button :disabled="importing">
              Select CSV File
            </n-button>
          </n-upload>

          <n-space v-if="selectedFile">
            <n-tag type="success">
              Selected: {{ selectedFile.name }} ({{ formatFileSize(selectedFile.size) }})
            </n-tag>
          </n-space>

          <n-button
            v-if="selectedFile"
            type="primary"
            size="large"
            :loading="importing"
            @click="handleImport"
            block
          >
            {{ importing ? 'Importing...' : 'Import Books' }}
          </n-button>
        </n-space>
      </n-card>

      <!-- Progress Section -->
      <n-card v-if="importing || importResult" title="Import Progress" :bordered="true">
        <n-space vertical size="large">
          <n-spin v-if="importing" size="large">
            <template #description>
              Importing your books... This may take a moment.
            </template>
          </n-spin>

          <div v-if="importResult && !importing">
            <!-- Summary Statistics -->
            <n-space vertical size="large">
              <n-alert
                :type="importResult.summary.failed_imports === 0 ? 'success' : 'warning'"
                :title="importResult.summary.failed_imports === 0 ? 'Import Completed Successfully!' : 'Import Completed with Some Errors'"
              >
                <n-space>
                  <n-statistic label="Total Rows" :value="importResult.summary.total_rows" />
                  <n-statistic label="Successful" :value="importResult.summary.successful_imports" />
                  <n-statistic label="Failed" :value="importResult.summary.failed_imports" />
                  <n-statistic label="Books Created" :value="importResult.summary.books_created" />
                  <n-statistic label="Books Updated" :value="importResult.summary.books_updated" />
                  <n-statistic label="Readings Created" :value="importResult.summary.readings_created" />
                </n-space>
              </n-alert>

              <!-- Errors Section -->
              <n-card v-if="importResult.errors.length > 0" title="Errors" size="small" :bordered="false">
                <n-list bordered>
                  <n-list-item v-for="error in importResult.errors" :key="error.row_number">
                    <n-thing>
                      <template #header>
                        Row {{ error.row_number }}
                        <span v-if="error.book_title"> - {{ error.book_title }}</span>
                      </template>
                      <template #description>
                        <n-text type="error">{{ error.error }}</n-text>
                      </template>
                    </n-thing>
                  </n-list-item>
                </n-list>
              </n-card>

              <!-- Success Section (Collapsed by default) -->
              <n-collapse v-if="importResult.successes.length > 0">
                <n-collapse-item title="View Successfully Imported Books" :name="1">
                  <n-list bordered>
                    <n-list-item v-for="success in importResult.successes" :key="success.row_number">
                      <n-thing>
                        <template #header>
                          {{ success.book_title }}
                        </template>
                        <template #description>
                          <n-space size="small">
                            <n-tag :type="success.operation === 'created' ? 'success' : 'info'" size="small">
                              {{ success.operation === 'created' ? 'Created' : 'Updated' }}
                            </n-tag>
                            <n-text depth="3">Row {{ success.row_number }}</n-text>
                          </n-space>
                        </template>
                      </n-thing>
                    </n-list-item>
                  </n-list>
                </n-collapse-item>
              </n-collapse>

              <!-- Actions -->
              <n-space justify="end">
                <n-button @click="resetImport">Import Another File</n-button>
                <n-button type="primary" @click="$router.push('/books')">
                  View My Books
                </n-button>
              </n-space>
            </n-space>
          </div>
        </n-space>
      </n-card>
    </n-space>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useMessage } from 'naive-ui'
import { useBooksStore } from '@/store/books'

const message = useMessage()
const booksStore = useBooksStore()

const selectedFile = ref(null)
const importing = ref(false)
const importResult = ref(null)

function handleFileChange(options) {
  if (options.fileList.length > 0) {
    selectedFile.value = options.fileList[0].file
  } else {
    selectedFile.value = null
  }
}

// Custom upload handler to prevent auto-upload
function handleUpload({ file, onFinish, onError }) {
  selectedFile.value = file.file
  onFinish()
}

async function handleImport() {
  if (!selectedFile.value) {
    message.error('Please select a CSV file first')
    return
  }

  // Validate file type
  if (!selectedFile.value.name.endsWith('.csv')) {
    message.error('Please select a valid CSV file')
    return
  }

  // Validate file size (10MB max)
  if (selectedFile.value.size > 10 * 1024 * 1024) {
    message.error('File size must be less than 10MB')
    return
  }

  importing.value = true
  importResult.value = null

  try {
    const result = await booksStore.importGoodreadsCSV(selectedFile.value)
    importResult.value = result

    if (result.summary.failed_imports === 0) {
      message.success(
        `Successfully imported ${result.summary.successful_imports} books!`
      )
    } else {
      message.warning(
        `Imported ${result.summary.successful_imports} books with ${result.summary.failed_imports} errors`
      )
    }
  } catch (error) {
    message.error(error.response?.data?.error || 'Failed to import CSV file')
    console.error('Import error:', error)
  } finally {
    importing.value = false
  }
}

function resetImport() {
  selectedFile.value = null
  importResult.value = null
}

function formatFileSize(bytes) {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(2) + ' MB'
}
</script>
