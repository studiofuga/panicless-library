<template>
  <div style="max-width: 400px; margin: 0 auto;">
    <n-card title="Login">
      <n-form ref="formRef" :model="formValue" :rules="rules">
        <n-form-item path="username" label="Username">
          <n-input v-model:value="formValue.username" placeholder="Enter username" />
        </n-form-item>
        <n-form-item path="password" label="Password">
          <n-input
            v-model:value="formValue.password"
            type="password"
            placeholder="Enter password"
            @keyup.enter="handleLogin"
          />
        </n-form-item>
        <n-space vertical>
          <n-button type="primary" :loading="loading" @click="handleLogin" block>
            Login
          </n-button>
          <n-text depth="3">
            Don't have an account?
            <n-button text type="primary" @click="$router.push('/register')">
              Register here
            </n-button>
          </n-text>
        </n-space>
      </n-form>
    </n-card>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/store/auth'
import { useMessage } from 'naive-ui'
import { NCard, NForm, NFormItem, NInput, NButton, NSpace, NText } from 'naive-ui'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()
const message = useMessage()

const formRef = ref(null)
const loading = ref(false)
const formValue = ref({
  username: '',
  password: ''
})

const rules = {
  username: [
    { required: true, message: 'Username is required', trigger: 'blur' }
  ],
  password: [
    { required: true, message: 'Password is required', trigger: 'blur' },
    { min: 8, message: 'Password must be at least 8 characters', trigger: 'blur' }
  ]
}

const handleLogin = async () => {
  try {
    await formRef.value?.validate()
    loading.value = true

    await authStore.login(formValue.value)

    message.success('Login successful!')

    const redirect = route.query.redirect || '/dashboard'
    router.push(redirect)
  } catch (error) {
    if (error.response) {
      message.error(error.response.data.message || 'Login failed')
    } else if (error.errors) {
      // Validation errors
      return
    } else {
      message.error('Login failed. Please try again.')
    }
  } finally {
    loading.value = false
  }
}
</script>
