<template>
  <div style="max-width: 400px; margin: 0 auto;">
    <n-card title="Register">
      <n-form ref="formRef" :model="formValue" :rules="rules">
        <n-form-item path="username" label="Username">
          <n-input v-model:value="formValue.username" placeholder="Choose a username" />
        </n-form-item>
        <n-form-item path="email" label="Email">
          <n-input v-model:value="formValue.email" placeholder="Enter your email" />
        </n-form-item>
        <n-form-item path="full_name" label="Full Name (Optional)">
          <n-input v-model:value="formValue.full_name" placeholder="Enter your full name" />
        </n-form-item>
        <n-form-item path="password" label="Password">
          <n-input v-model:value="formValue.password" type="password" placeholder="Choose a password" />
        </n-form-item>
        <n-space vertical>
          <n-button type="primary" :loading="loading" @click="handleRegister" block>
            Register
          </n-button>
          <n-text depth="3">
            Already have an account?
            <n-button text type="primary" @click="$router.push('/login')">
              Login here
            </n-button>
          </n-text>
        </n-space>
      </n-form>
    </n-card>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/store/auth'
import { useMessage } from 'naive-ui'
import { NCard, NForm, NFormItem, NInput, NButton, NSpace, NText } from 'naive-ui'

const router = useRouter()
const authStore = useAuthStore()
const message = useMessage()

const formRef = ref(null)
const loading = ref(false)
const formValue = ref({
  username: '',
  email: '',
  full_name: '',
  password: ''
})

const rules = {
  username: [
    { required: true, message: 'Username is required', trigger: 'blur' },
    { min: 3, max: 50, message: 'Username must be 3-50 characters', trigger: 'blur' }
  ],
  email: [
    { required: true, message: 'Email is required', trigger: 'blur' },
    { type: 'email', message: 'Please enter a valid email', trigger: 'blur' }
  ],
  password: [
    { required: true, message: 'Password is required', trigger: 'blur' },
    { min: 8, message: 'Password must be at least 8 characters', trigger: 'blur' }
  ]
}

const handleRegister = async () => {
  try {
    await formRef.value?.validate()
    loading.value = true

    const data = { ...formValue.value }
    if (!data.full_name) delete data.full_name

    await authStore.register(data)

    message.success('Registration successful!')
    router.push('/dashboard')
  } catch (error) {
    if (error.response) {
      message.error(error.response.data.message || 'Registration failed')
    } else if (error.errors) {
      return
    } else {
      message.error('Registration failed. Please try again.')
    }
  } finally {
    loading.value = false
  }
}
</script>
