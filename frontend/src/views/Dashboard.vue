<template>
  <div>
    <h1>Dashboard</h1>
    <p v-if="currentUser">Welcome back, {{ currentUser.full_name || currentUser.username }}!</p>

    <n-space vertical size="large">
      <!-- Statistics -->
      <n-card title="Reading Statistics">
        <n-spin :show="loadingStats">
          <n-space v-if="stats">
            <n-statistic label="Total Books Read" :value="stats.total_books_read" />
            <n-statistic label="Total Readings" :value="stats.total_readings" />
            <n-statistic label="Currently Reading" :value="stats.current_readings" />
            <n-statistic label="Completed" :value="stats.completed_readings" />
            <n-statistic
              v-if="stats.average_rating"
              label="Average Rating"
              :value="stats.average_rating.toFixed(1)"
            />
          </n-space>
        </n-spin>
      </n-card>

      <!-- Books by Year -->
      <n-card title="Books Read by Year" v-if="stats && stats.books_by_year && stats.books_by_year.length > 0">
        <n-list>
          <n-list-item v-for="yearStat in stats.books_by_year" :key="yearStat.year">
            <n-thing :title="`${yearStat.year}`" :description="`${yearStat.count} books`" />
          </n-list-item>
        </n-list>
      </n-card>

      <!-- Quick Actions -->
      <n-card title="Quick Actions">
        <n-space>
          <n-button type="primary" @click="$router.push('/books')">
            Browse Books
          </n-button>
          <n-button type="primary" @click="$router.push('/readings')">
            View Readings
          </n-button>
        </n-space>
      </n-card>
    </n-space>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useAuthStore } from '@/store/auth'
import { useReadingsStore } from '@/store/readings'
import { useMessage } from 'naive-ui'
import {
  NCard,
  NSpace,
  NButton,
  NStatistic,
  NSpin,
  NList,
  NListItem,
  NThing
} from 'naive-ui'

const authStore = useAuthStore()
const readingsStore = useReadingsStore()
const message = useMessage()

const currentUser = computed(() => authStore.currentUser)
const stats = computed(() => readingsStore.stats)
const loadingStats = ref(false)

onMounted(async () => {
  loadingStats.value = true
  try {
    await readingsStore.fetchStats()
  } catch (error) {
    message.error('Failed to load statistics')
  } finally {
    loadingStats.value = false
  }
})
</script>
