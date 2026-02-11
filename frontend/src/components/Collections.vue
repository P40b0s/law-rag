<template lang="pug">
.collections
  n-card(title="Управление коллекциями")
    template(#header-extra)
      n-button(
        type="primary"
        @click="showAddModal = true"
        size="small"
      )
        template(#icon)
          n-icon(:component="AddCircleOutline")
        | Добавить коллекцию

    n-spin(:show="loading")
      n-data-table(
        :columns="columns"
        :data="collections"
        :pagination="false"
        :bordered="false"
      )

  // Модальное окно для добавления коллекции
  n-modal(
    v-model:show="showAddModal"
    preset="dialog"
    title="Добавить коллекцию"
    positive-text="Добавить"
    negative-text="Отмена"
    @positive-click="handleAddCollection"
  )
    n-form(
      ref="addFormRef"
      :model="addFormData"
      :rules="addFormRules"
    )
      n-form-item(label="Название" path="name")
        n-input(
          v-model:value="addFormData.name"
          placeholder="Введите название коллекции"
        )
      n-form-item(label="Описание" path="description")
        n-input(
          v-model:value="addFormData.description"
          type="textarea"
          placeholder="Введите описание коллекции"
          :rows="3"
        )
      n-form-item(label="Ключевые слова")
        n-dynamic-tags(
          v-model:value="addFormData.keywords"
        )

  // Модальное окно для редактирования коллекции
  n-modal(
    v-model:show="showEditModal"
    preset="dialog"
    title="Редактировать коллекцию"
    positive-text="Сохранить"
    negative-text="Отмена"
    @positive-click="handleUpdateCollection"
  )
    n-form(
      ref="editFormRef"
      :model="editFormData"
      :rules="editFormRules"
    )
      n-form-item(label="Название")
        n-input(
          :value="editFormData.name"
          disabled
        )
      n-form-item(label="Описание" path="description")
        n-input(
          v-model:value="editFormData.description"
          type="textarea"
          placeholder="Введите описание коллекции"
          :rows="3"
        )
      n-form-item(label="Ключевые слова")
        n-dynamic-tags(
          v-model:value="editFormData.keywords"
        )
</template>

<script setup lang="ts">
import { ref, h } from 'vue'
import { NCard, NButton, NDataTable, NModal, NForm, NFormItem, NInput, NIcon, NSpin, NPopconfirm, NSpace, NDynamicTags, NTag, type DataTableColumns, type FormInst, type FormRules } from 'naive-ui'
import { AddCircleOutline, CreateOutline, TrashOutline } from '@vicons/ionicons5'
import useCollections from '@/composables/useCollections'
import { type Collection } from '@/types/collection'

// Composable
const {
  collections,
  loading,
  addCollection,
  updateCollection,
  deleteCollection
} = useCollections()

// Модальные окна
const showAddModal = ref(false)
const showEditModal = ref(false)

// Формы
const addFormRef = ref<FormInst | null>(null)
const editFormRef = ref<FormInst | null>(null)

const addFormData = ref({
  name: '',
  description: '',
  keywords: [] as string[]
})

const editFormData = ref({
  id: '',
  name: '',
  description: '',
  keywords: [] as string[]
})

// Правила валидации
const addFormRules: FormRules = {
  name: [
    { required: true, message: 'Введите название коллекции', trigger: 'blur' },
    { min: 2, message: 'Название должно содержать минимум 2 символа', trigger: 'blur' },
    {
      validator: (_rule, value: string) => {
        // Проверка на английский язык и отсутствие пробелов
        const validPattern = /^[a-zA-Z0-9_-]+$/
        if (!validPattern.test(value)) {
          return new Error('Название должно содержать только английские буквы, цифры, дефис и подчеркивание (без пробелов)')
        }
        return true
      },
      trigger: 'blur'
    }
  ],
  description: [
    { required: true, message: 'Введите описание коллекции', trigger: 'blur' }
  ]
}

const editFormRules: FormRules = {
  description: [
    { required: true, message: 'Введите описание коллекции', trigger: 'blur' }
  ]
}

// Обработчики
const handleAddCollection = async () => {
  try {
    await addFormRef.value?.validate()
    const newCollection = await addCollection(addFormData.value.name, addFormData.value.description, addFormData.value.keywords)
    if (newCollection) {
      showAddModal.value = false
      addFormData.value = { name: '', description: '', keywords: [] }
    }
  } catch (error) {
    console.error('Validation failed:', error)
    return false
  }
}

const handleEditClick = (collection: Collection) => {
  editFormData.value = {
    id: collection.id,
    name: collection.name,
    description: collection.description,
    keywords: [...collection.keywords]
  }
  showEditModal.value = true
}

const handleUpdateCollection = async () => {
  try {
    await editFormRef.value?.validate()
    const success = await updateCollection(editFormData.value.id, editFormData.value.description, editFormData.value.keywords)
    if (success) {
      showEditModal.value = false
    }
  } catch (error) {
    console.error('Validation failed:', error)
    return false
  }
}

const handleDeleteCollection = async (id: string) => {
  await deleteCollection(id)
}

// Колонки таблицы
const columns: DataTableColumns<Collection> = [
  {
    title: 'Название',
    key: 'name',
    width: 200
  },
  {
    title: 'Описание',
    key: 'description',
    ellipsis: {
      tooltip: true
    }
  },
  {
    title: 'Ключевые слова',
    key: 'keywords',
    width: 250,
    render: (row) => {
      return h(NSpace, { size: 4 }, () =>
        row.keywords.map((keyword: string) =>
          h(NTag, { size: 'small', type: 'info' }, { default: () => keyword })
        )
      )
    }
  },
  {
    title: 'Действия',
    key: 'actions',
    width: 150,
    render: (row) => {
      return h(NSpace, { size: 8 }, () => [
        h(
          NButton,
          {
            size: 'small',
            type: 'primary',
            secondary: true,
            onClick: () => handleEditClick(row)
          },
          {
            icon: () => h(NIcon, { component: CreateOutline })
          }
        ),
        h(
          NPopconfirm,
          {
            onPositiveClick: () => handleDeleteCollection(row.id)
          },
          {
            trigger: () => h(
              NButton,
              {
                size: 'small',
                type: 'error',
                secondary: true
              },
              {
                icon: () => h(NIcon, { component: TrashOutline })
              }
            ),
            default: () => 'Вы уверены, что хотите удалить эту коллекцию? Все документы в ней также будут удалены.'
          }
        )
      ])
    }
  }
]
</script>

<style scoped lang="scss">
.collections {
  width: 100%;
}
</style>
