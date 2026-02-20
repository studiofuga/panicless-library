<template>
  <div>
    <n-space justify="space-between" style="margin-bottom: 24px;">
      <h1>User Management</h1>
      <n-button type="primary" @click="showInviteModal = true">
        Invite User
      </n-button>
    </n-space>

    <n-spin :show="loading">
      <n-data-table
        :columns="columns"
        :data="users"
        :row-key="row => row.id"
        :bordered="true"
      />
    </n-spin>

    <!-- Invite User Modal -->
    <n-modal v-model:show="showInviteModal">
      <n-card
        style="width: 500px"
        title="Invite User"
        :bordered="false"
        size="huge"
        role="dialog"
        aria-modal="true"
      >
        <n-form ref="inviteFormRef" :model="inviteForm" :rules="inviteRules">
          <n-form-item label="Username" path="username">
            <n-input v-model:value="inviteForm.username" placeholder="Username" />
          </n-form-item>
          <n-form-item label="Email" path="email">
            <n-input v-model:value="inviteForm.email" placeholder="Email address" />
          </n-form-item>
          <n-form-item label="Full Name (Optional)" path="full_name">
            <n-input v-model:value="inviteForm.full_name" placeholder="Full name" />
          </n-form-item>
          <n-form-item label="Role" path="role">
            <n-select v-model:value="inviteForm.role" :options="roleOptions" />
          </n-form-item>
        </n-form>
        <template #footer>
          <n-space justify="end">
            <n-button @click="showInviteModal = false">Cancel</n-button>
            <n-button type="primary" :loading="saving" @click="handleInvite">
              Send Invitation
            </n-button>
          </n-space>
        </template>
      </n-card>
    </n-modal>

    <!-- Invitation Token Modal -->
    <n-modal v-model:show="showTokenModal">
      <n-card
        style="width: 500px"
        title="Invitation Created"
        :bordered="false"
        size="huge"
        role="dialog"
        aria-modal="true"
      >
        <n-space vertical :size="16">
          <n-alert type="success" :closable="false">
            User created successfully. Share the following invitation link:
          </n-alert>
          <n-input
            :value="invitationLink"
            readonly
            type="textarea"
            :rows="2"
          />
          <n-button @click="copyInvitationLink" block>
            Copy Link
          </n-button>
        </n-space>
        <template #footer>
          <n-space justify="end">
            <n-button @click="showTokenModal = false">Close</n-button>
          </n-space>
        </template>
      </n-card>
    </n-modal>

    <!-- Edit User Modal -->
    <n-modal v-model:show="showEditModal">
      <n-card
        style="width: 500px"
        title="Edit User"
        :bordered="false"
        size="huge"
        role="dialog"
        aria-modal="true"
      >
        <n-form ref="editFormRef" :model="editForm">
          <n-form-item label="Email">
            <n-input v-model:value="editForm.email" placeholder="Email address" />
          </n-form-item>
          <n-form-item label="Full Name">
            <n-input v-model:value="editForm.full_name" placeholder="Full name" />
          </n-form-item>
          <n-form-item label="Role">
            <n-select v-model:value="editForm.role" :options="roleOptions" />
          </n-form-item>
          <n-form-item label="Enabled">
            <n-switch v-model:value="editForm.enabled" />
          </n-form-item>
        </n-form>
        <template #footer>
          <n-space justify="end">
            <n-button @click="showEditModal = false">Cancel</n-button>
            <n-button type="primary" :loading="saving" @click="handleUpdate">
              Save Changes
            </n-button>
          </n-space>
        </template>
      </n-card>
    </n-modal>
  </div>
</template>

<script setup>
import { ref, computed, h, onMounted } from 'vue'
import { useAdminStore } from '@/store/admin'
import { useMessage, useDialog } from 'naive-ui'
import {
  NSpace,
  NButton,
  NInput,
  NSpin,
  NDataTable,
  NModal,
  NCard,
  NForm,
  NFormItem,
  NSelect,
  NSwitch,
  NTag,
  NAlert
} from 'naive-ui'

const adminStore = useAdminStore()
const message = useMessage()
const dialog = useDialog()

const users = computed(() => adminStore.users)
const loading = computed(() => adminStore.loading)

const showInviteModal = ref(false)
const showTokenModal = ref(false)
const showEditModal = ref(false)
const saving = ref(false)
const invitationToken = ref('')
const editingUserId = ref(null)

const roleOptions = [
  { label: 'User', value: 'user' },
  { label: 'Admin', value: 'admin' }
]

const inviteForm = ref({
  username: '',
  email: '',
  full_name: '',
  role: 'user'
})

const inviteRules = {
  username: [
    { required: true, message: 'Username is required', trigger: 'blur' },
    { min: 3, max: 50, message: 'Username must be 3-50 characters', trigger: 'blur' }
  ],
  email: [
    { required: true, message: 'Email is required', trigger: 'blur' },
    { type: 'email', message: 'Please enter a valid email', trigger: 'blur' }
  ]
}

const editForm = ref({
  email: '',
  full_name: '',
  role: 'user',
  enabled: true
})

const invitationLink = computed(() => {
  if (!invitationToken.value) return ''
  const base = window.location.origin
  return `${base}/complete-registration?token=${invitationToken.value}`
})

const columns = [
  { title: 'Username', key: 'username', sorter: 'default' },
  { title: 'Email', key: 'email' },
  { title: 'Full Name', key: 'full_name', render: (row) => row.full_name || '-' },
  {
    title: 'Role',
    key: 'role',
    render: (row) => h(NTag, { type: row.role?.toLowerCase() === 'admin' ? 'warning' : 'info', size: 'small' }, () => row.role)
  },
  {
    title: 'Enabled',
    key: 'enabled',
    render: (row) => h(NTag, { type: row.enabled ? 'success' : 'error', size: 'small' }, () => row.enabled ? 'Yes' : 'No')
  },
  {
    title: 'Created',
    key: 'created_at',
    render: (row) => row.created_at ? new Date(row.created_at).toLocaleDateString() : '-'
  },
  {
    title: 'Actions',
    key: 'actions',
    render: (row) => {
      const buttons = [
        h(NButton, { size: 'small', onClick: () => openEditModal(row) }, () => 'Edit'),
        h(NButton, { size: 'small', type: 'error', onClick: () => confirmDelete(row) }, () => 'Delete')
      ]
      if (!row.enabled) {
        buttons.push(
          h(NButton, { size: 'small', type: 'warning', onClick: () => handleResendInvitation(row.id) }, () => 'Resend Invite')
        )
      }
      return h(NSpace, { size: 'small' }, () => buttons)
    }
  }
]

const openEditModal = (user) => {
  editingUserId.value = user.id
  editForm.value = {
    email: user.email,
    full_name: user.full_name || '',
    role: user.role,
    enabled: user.enabled
  }
  showEditModal.value = true
}

const confirmDelete = (user) => {
  dialog.warning({
    title: 'Delete User',
    content: `Are you sure you want to delete user "${user.username}"?`,
    positiveText: 'Delete',
    negativeText: 'Cancel',
    onPositiveClick: async () => {
      try {
        await adminStore.deleteUser(user.id)
        message.success('User deleted')
      } catch {
        message.error('Failed to delete user')
      }
    }
  })
}

const handleInvite = async () => {
  try {
    await inviteFormRef.value?.validate()
    saving.value = true

    const data = { ...inviteForm.value }
    if (!data.full_name) delete data.full_name

    const result = await adminStore.createUser(data)
    invitationToken.value = result.invitation_token
    showInviteModal.value = false
    showTokenModal.value = true

    // Reset form
    inviteForm.value = { username: '', email: '', full_name: '', role: 'user' }
  } catch (error) {
    if (error.response) {
      message.error(error.response.data.message || 'Failed to create user')
    } else if (error.errors) {
      return
    } else {
      message.error('Failed to create user')
    }
  } finally {
    saving.value = false
  }
}

const handleUpdate = async () => {
  try {
    saving.value = true
    await adminStore.updateUser(editingUserId.value, editForm.value)
    message.success('User updated')
    showEditModal.value = false
  } catch (error) {
    message.error(error.response?.data?.message || 'Failed to update user')
  } finally {
    saving.value = false
  }
}

const handleResendInvitation = async (id) => {
  try {
    const result = await adminStore.resendInvitation(id)
    invitationToken.value = result.invitation_token
    showTokenModal.value = true
    message.success('Invitation resent')
  } catch {
    message.error('Failed to resend invitation')
  }
}

const copyInvitationLink = async () => {
  try {
    await navigator.clipboard.writeText(invitationLink.value)
    message.success('Link copied to clipboard')
  } catch {
    message.error('Failed to copy link')
  }
}

const inviteFormRef = ref(null)

onMounted(async () => {
  try {
    await adminStore.fetchUsers()
  } catch {
    message.error('Failed to load users')
  }
})
</script>
