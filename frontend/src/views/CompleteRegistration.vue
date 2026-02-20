<template>
  <div style="max-width: 400px; margin: 0 auto;">
    <n-card title="Complete Registration">
      <n-alert v-if="!token" type="error" :closable="false" style="margin-bottom: 16px;">
        Invalid or missing invitation token. Please use the link provided in your invitation.
      </n-alert>
      <n-form v-else ref="formRef" :model="formValue" :rules="rules">
        <n-form-item path="password" label="Password">
          <n-input v-model:value="formValue.password" type="password" placeholder="Choose a password (min 8 characters)" />
        </n-form-item>
        <n-form-item path="confirm_password" label="Confirm Password">
          <n-input v-model:value="formValue.confirm_password" type="password" placeholder="Confirm your password" />
        </n-form-item>
        <n-form-item path="full_name" label="Full Name (Optional)">
          <n-input v-model:value="formValue.full_name" placeholder="Enter your full name" />
        </n-form-item>
        <n-space vertical>
          <n-button type="primary" :loading="loading" @click="handleComplete" block>
            Complete Registration
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
import { ref, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/store/auth'
import { useMessage } from 'naive-ui'
import { NCard, NForm, NFormItem, NInput, NButton, NSpace, NText, NAlert } from 'naive-ui'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()
const message = useMessage()

const token = computed(() => route.query.token || '')
const formRef = ref(null)
const loading = ref(false)
const formValue = ref({
  password: '',
  confirm_password: '',
  full_name: ''
})

const rules = {
  password: [
    { required: true, message: 'Password is required', trigger: 'blur' },
    { min: 8, message: 'Password must be at least 8 characters', trigger: 'blur' }
  ],
  confirm_password: [
    { required: true, message: 'Please confirm your password', trigger: 'blur' },
    {
      validator: (rule, value) => value === formValue.value.password,
      message: 'Passwords do not match',
      trigger: 'blur'
    }
  ]
}

const handleComplete = async () => {
  try {
    await formRef.value?.validate()
    loading.value = true

    const data = {
      invitation_token: token.value,
      password: formValue.value.password
    }
    if (formValue.value.full_name) {
      data.full_name = formValue.value.full_name
    }

    await authStore.completeRegistration(data)

    message.success('Registration completed successfully!')
    router.push('/statistics')
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
