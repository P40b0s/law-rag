<template lang="pug">
n-card.document-manager(title="Управление документами")
  template(#header-extra)
    n-button(type="primary" @click="openAddModal")
      template(#icon)
        n-icon: add-icon
      | Добавить документ

  n-space(vertical :size="20")
    //- Список документов
    n-list(bordered)
      n-list-item(v-for="document in documents" :key="document.date + document.number")
        template(#suffix)
          n-space
            n-tooltip(trigger="hover") Удалить документ
              template(#trigger)
                n-button(round text @click="confirmDelete(document)")
                  template(#icon)
                    n-icon(:size="25" color="#ec3c36"): trash-bin
        n-thing(:title="`${document.title || 'Документ'} (${document.number})`" :description="`Дата: ${formatDate(document.sign_date)} | Статус: ${getStatusText(document.status)}`")
          template(#footer v-if="document.first_chunk")
            n-text(depth="3" :style="{ fontSize: '12px' }") 
              | Хэш: {{ document.first_chunk.hash.slice(0, 16) }}...
            .document-preview(v-if="document.first_chunk")
              n-text(depth="3" :style="{ fontSize: '12px', display: 'block', marginTop: '4px' }") 
                | Фрагмент: {{ truncateContent(document.first_chunk.content, 100) }}

    //- Пустое состояние
    n-empty(
      v-if="documents.length === 0"
      description="Нет документов"
    )
      template(#extra)
        n-button(size="small" @click="openAddModal") Добавить первый документ

//- Модальное окно добавления документа
n-modal(
  v-model:show="showModal"
  title="Добавить документ"
  preset="dialog"
  :mask-closable="false"
  positive-text="Добавить"
  negative-text="Отмена"
  @positive-click="handleSave"
  @negative-click="handleCancel"
)
  n-form(
    ref="formRef"
    :model="formModel"
    :rules="formRules"
    label-placement="top"
    :style="{ padding: '10px' }"
  )
    n-form-item(label="Номер документа" path="number" required)
      n-input(
        v-model:value="formModel.number"
        placeholder="Введите номер документа (например, 123-ФЗ)"
        :maxlength="50"
      )
    
    n-form-item(label="Дата подписания" path="sign_date" required)
      n-date-picker(
        v-model:value="formModel.sign_date"
        type="date"
        clearable
        placeholder="Выберите дату подписания"
        :is-date-disabled="disableFutureDate"
        value-format="yyyy-MM-dd"
        format="dd.MM.yyyy"
        :style="{ width: '100%' }"
      )
    
    n-alert(type="info" :style="{ marginTop: '10px' }")
      | Остальные данные документа будут автоматически загружены с сервера

//- Диалог подтверждения удаления
n-modal(
  v-model:show="showDeleteConfirm"
  preset="dialog"
  type="error"
  title="Подтверждение удаления"
  :content="`Вы уверены, что хотите удалить документ ${documentToDelete?.number}?`"
  positive-text="Удалить"
  negative-text="Отмена"
  @positive-click="handleDelete"
  @negative-click="showDeleteConfirm = false"
)

//- Модальное окно деталей документа
n-modal(
  v-model:show="showDetailsModal"
  title="Детали документа"
  preset="dialog"
  :mask-closable="false"
  :bordered="false"
  :style="{ width: '800px', maxWidth: '90vw' }"
)
  n-card(v-if="selectedDocument" size="small")
    template(#header)
      n-space(vertical :size="8")
        n-h2(:style="{ margin: 0 }") {{ selectedDocument.title || 'Документ' }}
        n-text(depth="3") № {{ selectedDocument.number }} от {{ formatDate(selectedDocument.sign_date) }}
    
    n-space(vertical :size="16")
      n-descriptions(label-placement="left" bordered :column="1")
        n-descriptions-item(label="Хэш документа") {{ selectedDocument.hash }}
        n-descriptions-item(label="Статус") 
          n-tag(:type="getStatusTagType(selectedDocument.status)") {{ getStatusText(selectedDocument.status) }}
        n-descriptions-item(label="Путь к файлу") {{ selectedDocument.path || 'Не указан' }}
        n-descriptions-item(label="Ссылка на публикацию" v-if="selectedDocument.publication_url")
          n-a(:href="selectedDocument.publication_url" target="_blank") {{ selectedDocument.publication_url }}
      
      n-card(title="Содержание" size="small" v-if="selectedDocument.first_chunk?.content")
        n-text(pre :style="{ whiteSpace: 'pre-wrap', wordBreak: 'break-word' }") {{ selectedDocument.first_chunk.content }}
      
      n-card(title="Метаданные" size="small" v-if="selectedDocument.first_chunk?.meta")
        n-descriptions(label-placement="left" bordered :column="1")
          n-descriptions-item(label="Индекс чанка") {{ selectedDocument.first_chunk.meta.chunk_index }}
          n-descriptions-item(label="Количество токенов") {{ selectedDocument.first_chunk.meta.token_count }}
</template>

<script lang="ts">
import { ref, computed, onMounted } from 'vue'
import {
  NCard,
  NButton,
  NIcon,
  NSpace,
  NInput,
  NList,
  NListItem,
  NThing,
  NAvatar,
  NEmpty,
  NModal,
  NForm,
  NFormItem,
  NAlert,
  NTag,
  NDescriptions,
  NDescriptionsItem,
  NText,
  NH2,
  NA,
  NDatePicker,
  NInputNumber,
  NDynamicTags,
  type FormRules,
  type FormInst
} from 'naive-ui'
import { notify_service } from '@/services/notification_service'
import { AddOutline as AddIcon, SearchOutline as SearchIcon, TrashBin as TrashBinIcon, EyeOutline as ViewIcon } from '@vicons/ionicons5'
import { DocumentText as DocIcon } from '@vicons/ionicons5'
import type { Chunk, Document, LoadStatus } from '../types/document'
import { DateFormat, DateTime, to_rfc3339 } from '@/services/date'

</script>

<script lang="ts" setup>

const formRef = ref<FormInst | null>(null)

// Состояние
const documents = ref<Document[]>([])
const showModal = ref(false)
const showDeleteConfirm = ref(false)
const showDetailsModal = ref(false)
const documentToDelete = ref<Document | null>(null)
const selectedDocument = ref<Document | null>(null)

// Модель формы (только номер и дата)
const formModel = ref({
  number: '',
  sign_date: null as number | null
})

// Правила валидации
const formRules: FormRules = {
  number: [
    {
      required: true,
      message: 'Номер документа обязателен',
      trigger: ['blur', 'input']
    },
    {
      min: 2,
      message: 'Номер должен содержать минимум 2 символа',
      trigger: ['blur', 'input']
    }
  ],
  sign_date: [
    {
      required: true,
      message: 'Дата подписания обязательна',
      trigger: ['blur', 'change'],
      validator: (_, value) => !!value
    }
  ]
}

// Загрузка документов (заглушка)
onMounted(async () => {
  // Загрузка документов с бэкенда
  // const response = await fetchDocuments();
  // documents.value = response;
})

// Вспомогательные функции
const formatDate = (dateString: Date) => {
  try {
    const date = DateTime.parse(dateString)
    return date.to_string(DateFormat.DotDate);
  } catch {
    return dateString
  }
}

const truncateContent = (content: string, maxLength: number) => 
{
  if (!content) return ''
  return content.length > maxLength 
    ? content.substring(0, maxLength) + '...' 
    : content
}

const getStatusText = (status: LoadStatus) => {
  const statusMap = {
    'NotFound': 'Не найден',
    'Timeout': 'Таймаут',
    'Complete': 'Загружен',
    'Pending': 'В обработке'
  }
  return statusMap[status] || status
}

const getStatusColor = (status: LoadStatus) => {
  const colorMap = {
    'NotFound': '#ff4757',
    'Timeout': '#ffa502',
    'Complete': '#2ed573',
    'Pending': '#1e90ff'
  }
  return colorMap[status] || '#cccccc'
}

const getStatusTagType = (status: LoadStatus) => {
  const typeMap = {
    'NotFound': 'error',
    'Timeout': 'warning',
    'Complete': 'success',
    'Pending': 'info'
  }
  return typeMap[status] || 'default'
}

const disableFutureDate = (timestamp: number) => {
  return timestamp > Date.now()
}

// Открытие модального окна для добавления
const openAddModal = () => {
  formModel.value = {
    number: '',
    sign_date: null
  }
  showModal.value = true
}

// Просмотр деталей документа
const viewDocumentDetails = (document: Document) => {
  selectedDocument.value = document
  showDetailsModal.value = true
}

// Подготовка к удалению
const confirmDelete = (document: Document) => {
  documentToDelete.value = document
  showDeleteConfirm.value = true
}

// Удаление документа
const handleDelete = async () => {
  if (documentToDelete.value) {
    try {
      // Вызов API для удаления документа
      // await deleteDocument(documentToDelete.value.hash);
      
      // Удаление из локального состояния
      documents.value = documents.value.filter(
        doc => doc.first_chunk?.hash !== documentToDelete.value!.first_chunk?.hash
      )
      
      notify_service.success('Документ успешно удален')
    } catch (error) {
      notify_service.error('Ошибка при удалении документа')
      console.error('Delete error:', error)
    }
    documentToDelete.value = null
  }
  showDeleteConfirm.value = false
}

// Сохранение документа (только номер и дата)
const handleSave = () => {
  formRef.value?.validate(async (errors) => {
    if (errors) {
      notify_service.error('Пожалуйста, исправьте ошибки в форме')
      return false
    }

    try {
      // Создание базового объекта документа
      const dateStr = to_rfc3339(new Date(formModel.value.sign_date!))
      
      // Создаем временный документ со статусом Pending
      const newDocument: Document = {
        date: dateStr,
        number: formModel.value.number,
        first_chunk: null,
        status: 'Pending'
      }

      // Отправка на бэкенд (только номер и дата)
      // const response = await addDocument({
      //   number: formModel.value.number,
      //   sign_date: dateStr
      // });
      
      // Для демонстрации добавляем локально
      // В реальном приложении нужно дождаться ответа от бэкенда
      documents.value.unshift({
        ...newDocument,
      })

      notify_service.success('Документ добавлен. Данные загружаются...')
      showModal.value = false
      resetForm()
      return true
    } catch (error) {
      notify_service.error('Ошибка при добавлении документа')
      console.error('Save error:', error)
      return false
    }
  })
}

// Отмена
const handleCancel = () => {
  showModal.value = false
  resetForm()
}

// Сброс формы
const resetForm = () => {
  formModel.value = {
    number: '',
    sign_date: null
  }
}
</script>

<style lang="scss" scoped>
.document-manager {
  max-width: 800px;
  min-width: 600px;
  margin: 0 auto;
}

.n-list {
  max-height: calc(100vh - 220px);
  overflow-y: auto;
}

.n-list-item {
  padding: 16px;
  
  .n-thing {
    width: 100%;
  }
}

.document-preview {
  padding: 8px;
  background-color: #f8f9fa;
  border-radius: 4px;
  margin-top: 8px;
  border-left: 3px solid #3498db;
}

.n-avatar {
  font-weight: bold;
  display: flex;
  align-items: center;
  justify-content: center;
}

:deep(.n-modal-body-wrapper) {
  .n-card {
    margin: 0;
  }
}
</style>