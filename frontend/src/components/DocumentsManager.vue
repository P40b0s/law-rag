<template lang="pug">
.documents-manager
  n-card.search-card(title="Поиск документа" :bordered="false")
    n-form(
      ref="formRef"
      :model="formData"
      :rules="formRules"
      label-placement="top"
      require-mark-placement="right-hanging"
    )
      n-grid(:cols="2" :x-gap="16")
        n-form-item-gi(label="Дата подписания" path="sign_date")
          n-date-picker(
            v-model:value="formData.sign_date"
            type="date"
            placeholder="Выберите дату подписания"
            :style="{ width: '100%' }"
            format="dd.MM.yyyy"
            value-format="yyyy-MM-dd"
          )
        n-form-item-gi(label="Номер документа" path="number")
          n-input(
            v-model:value="formData.number"
            placeholder="Например: 123-ФЗ"
            clearable
          )
            template(#prefix)
              n-icon(:component="DocumentTextOutline")

      n-form-item(label="Коллекция" path="collection_id")
        n-select(
          v-model:value="formData.collection_id"
          :options="collectionOptions"
          placeholder="Выберите коллекцию"
          :loading="collectionsLoading"
          clearable
        )

      n-space.actions(:justify="'end'")
        n-button(
          type="primary"
          size="large"
          :loading="isLoading"
          :disabled="!formData.sign_date || !formData.number || !formData.collection_id"
          @click="handleRequestDocument"
        )
          template(#icon)
            n-icon(:component="SearchOutline")
          | Найти документ

  // Информация о документе
  n-card.document-info(
    v-if="currentDocument"
    :title="currentDocument.document_title"
    :bordered="false"
  )
    template(#header-extra)
      n-tag(:type="getStatusType(currentDocument.status)" :bordered="false" size="large")
        template(#icon)
          n-icon(:component="getStatusIcon(currentDocument.status)")
        | {{ getStatusLabel(currentDocument.status) }}

    n-space(vertical :size="16")
      // Основная информация
      n-descriptions(
        :column="2"
        :label-style="{ fontWeight: 'bold' }"
        bordered
      )
        n-descriptions-item(label="Номер документа")
          n-text(strong) {{ currentDocument.document_number }}
        n-descriptions-item(label="Дата подписания")
          n-text {{ formatDate(currentDocument.document_sign_date) }}
        n-descriptions-item(label="Хеш документа" :span="2")
          n-text(code) {{ currentDocument.document_hash }}
        n-descriptions-item(label="URI документа" :span="2")
          n-text(code) {{ currentDocument.document_uri }}

      // Статистика
      n-space(:size="16")
        n-statistic(label="Количество чанков" tabular-nums)
          template(#prefix)
            n-icon(:component="DocumentsOutline" size="20")
          | {{ currentDocument.chunks_count }}

        n-statistic(label="Эмбеддинги")
          template(#prefix)
            n-icon(:component="currentDocument.has_embeddings ? CheckmarkCircleOutline : CloseCircleOutline" size="20" :color="currentDocument.has_embeddings ? '#18a058' : '#d03050'")
          | {{ currentDocument.has_embeddings ? 'Созданы' : 'Не созданы' }}

      n-divider

      // Чанки документа
      .chunks-section(v-if="currentDocument.chunks && currentDocument.chunks.length > 0")
        n-text.section-title(strong) Чанки документа
        n-space(vertical :size="12" style="margin-top: 12px")
          n-card.chunk-card(
            v-for="(chunk, index) in currentDocument.chunks"
            :key="index"
            size="small"
            :bordered="false"
          )
            template(#header)
              n-space(align="center")
                n-icon(:component="DocumentOutline" size="18")
                n-text Чанк {{'# ' + chunk.meta?.chunk_index ?? index }}
                n-tag(
                  v-if="chunk.meta"
                  size="small"
                  :bordered="false"
                )
                  | {{ chunk.meta.token_count }} токенов

            n-ellipsis(
              :line-clamp="3"
              :tooltip="false"
            )
              n-text {{ chunk.content }}

      n-divider

      // Действия с документом
      n-space(:size="12")
        n-button(
          type="success"
          size="large"
          :loading="isEmbedding"
          :disabled="currentDocument.has_embeddings"
          @click="handleCreateEmbeddings"
        )
          template(#icon)
            n-icon(:component="LayersOutline")
          | {{ currentDocument.has_embeddings ? 'Эмбеддинги созданы' : 'Создать эмбеддинги' }}

        n-button(
          type="warning"
          size="large"
          @click="handleClearDocument"
        )
          template(#icon)
            n-icon(:component="RefreshOutline")
          | Загрузить другой документ
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  NCard,
  NForm,
  NFormItemGi,
  NFormItem,
  NGrid,
  NInput,
  NDatePicker,
  NButton,
  NSpace,
  NIcon,
  NTag,
  NSelect,
  NDescriptions,
  NDescriptionsItem,
  NText,
  NStatistic,
  NDivider,
  NEllipsis,
  type FormInst,
  type FormRules
} from 'naive-ui'
import {
  SearchOutline,
  DocumentTextOutline,
  DocumentOutline,
  DocumentsOutline,
  LayersOutline,
  RefreshOutline,
  CheckmarkCircleOutline,
  CloseCircleOutline,
  TimeOutline,
  CloudDoneOutline,
  AlertCircleOutline,
  HelpCircleOutline
} from '@vicons/ionicons5'
import useEmbeddings from '@/composables/useEmbeddings'
import useCollections from '@/composables/useCollections'
import useDocuments from '@/composables/useDocuments'
import { notify_service } from '@/services/notification_service'
import type { DocumentStatus } from '@/types/document'
import { DateTime } from '@/services/date'

// Composables
const { get_loading, get_embedding, request_document, embedding_document } = useEmbeddings()
const { collectionOptions, loading: collectionsLoading } = useCollections()
const { currentDocument, addOrUpdateDocument, clearCurrentDocument } = useDocuments()

// Reactive state
const formRef = ref<FormInst | null>(null)
const formData = ref({
  sign_date: null as number | null,
  number: '',
  collection_id: null as string | null
})

// Form rules
const formRules: FormRules = {
  sign_date: {
    type: 'number',
    required: true,
    message: 'Выберите дату подписания',
    trigger: 'change'
  },
  number: {
    required: true,
    message: 'Введите номер документа',
    trigger: ['input', 'blur']
  },
  collection_id: {
    type: 'string',
    required: true,
    message: 'Выберите коллекцию',
    trigger: 'change'
  }
}

// Computed
const isLoading = computed(() => get_loading().value)
const isEmbedding = computed(() => get_embedding().value)

// Methods
const handleRequestDocument = async () => {
  if (!formRef.value) return

  try {
    await formRef.value.validate()

    if (!formData.value.sign_date || !formData.value.number || !formData.value.collection_id) {
      notify_service.warning('Заполните все поля', 'Необходимо указать дату, номер и коллекцию')
      return
    }

    const date = new Date(formData.value.sign_date)
    const dateTime = DateTime.parse(date)

    const doc = await request_document(dateTime, formData.value.number, formData.value.collection_id)

    if (doc) {
      addOrUpdateDocument(doc)
      notify_service.success('Документ найден', 'Документ успешно загружен с сервера')
    }
  } catch (error) {
    console.error('Validation error:', error)
  }
}

const handleCreateEmbeddings = async () => {
  if (!currentDocument.value) return

  const success = await embedding_document(
    currentDocument.value.document_hash,
    currentDocument.value.collection_id
  )

  if (success) {
    notify_service.success('Эмбеддинги созданы', 'Эмбеддинги для документа успешно созданы')
  }
}

const handleClearDocument = () => {
  clearCurrentDocument()
  formData.value = {
    sign_date: null,
    number: '',
    collection_id: null
  }
}

const formatDate = (dateTime: any): string => {
  if (!dateTime) return 'Не указана'
  const { day, month, year } = dateTime
  return `${String(day).padStart(2, '0')}.${String(month).padStart(2, '0')}.${year}`
}

const getStatusLabel = (status: DocumentStatus): string => {
  const statusMap: Record<DocumentStatus, string> = {
    'NotLoaded': 'Не загружен',
    'Loaded': 'Загружен',
    'Embedded': 'Эмбеддинги созданы',
    'Unknown': 'Неизвестно'
  }
  return statusMap[status] || 'Неизвестно'
}

const getStatusType = (status: DocumentStatus): 'default' | 'success' | 'warning' | 'error' | 'info' => {
  const typeMap: Record<DocumentStatus, 'default' | 'success' | 'warning' | 'error' | 'info'> = {
    'NotLoaded': 'default',
    'Loaded': 'info',
    'Embedded': 'success',
    'Unknown': 'warning'
  }
  return typeMap[status] || 'default'
}

const getStatusIcon = (status: DocumentStatus) => {
  const iconMap: Record<DocumentStatus, any> = {
    'NotLoaded': TimeOutline,
    'Loaded': CloudDoneOutline,
    'Embedded': CheckmarkCircleOutline,
    'Unknown': HelpCircleOutline
  }
  return iconMap[status] || AlertCircleOutline
}
</script>

<style scoped>
.documents-manager {
  max-width: 1200px;
  margin: 0 auto;
  padding: 24px;
}

.search-card {
  margin-bottom: 24px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  border-radius: 12px;
}

.document-info {
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.12);
  border-radius: 12px;
  animation: slideIn 0.3s ease-out;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.actions {
  margin-top: 16px;
}

.chunks-section {
  margin-top: 16px;
}

.section-title {
  font-size: 16px;
  display: block;
  margin-bottom: 12px;
}

.chunk-card {
  background: linear-gradient(135deg, #f5f7fa 0%, #f0f2f5 100%);
  border-radius: 8px;
  transition: all 0.3s ease;
}

.chunk-card:hover {
  transform: translateX(4px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

:deep(.n-statistic) {
  padding: 12px 16px;
  background: #f8f9fa;
  border-radius: 8px;
}

:deep(.n-descriptions) {
  border-radius: 8px;
}

:deep(.n-button) {
  font-weight: 500;
  transition: all 0.3s ease;
}

:deep(.n-button:hover) {
  transform: translateY(-2px);
}
</style>
