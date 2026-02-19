<template lang="pug">
.documents-list
  n-modal(
    v-model:show="show_add_modal" 
    title="Добавление документа"
    :mask-closable="true"
     preset="card"
     :style="{ maxWidth: '600px' }"
     closable)
    document-add-form(@document-added="handleDocumentAdded")
  n-button(
    type="primary"
    @click="show_add_modal = true"
    style="margin-bottom: 16px"
  )
    template(#icon)
      n-icon(:component="AddOutline")
    | Добавить документ
  n-card(title="Список документов" :bordered="false")
    template(#header-extra)
      n-space(align="center")
        n-text(depth="3") Всего: {{ totalCount }}
        n-button(
          type="primary"
          size="small"
          @click="handleLoadDocuments"
          :loading="documentsLoading"
        )
          template(#icon)
            n-icon(:component="RefreshOutline")
          | Обновить

    n-space(vertical :size="16")
      // Фильтры и поиск
      n-space(:size="12")
        n-input(
          v-model:value="searchQuery"
          placeholder="Поиск по названию или номеру..."
          clearable
          style="width: 300px"
        )
          template(#prefix)
            n-icon(:component="SearchOutline")

        n-select(
          v-model:value="statusFilter"
          :options="statusOptions"
          placeholder="Фильтр по статусу"
          clearable
          style="width: 200px"
        )

        n-select(
          v-model:value="collectionFilter"
          :options="collectionOptions"
          placeholder="Фильтр по коллекции"
          clearable
          style="width: 200px"
        )

      // Таблица документов
      n-data-table(
        :columns="columns"
        :data="filteredDocuments"
        :loading="documentsLoading"
        :pagination="pagination"
        :row-key="(row) => row.document_hash"
        :bordered="false"
        striped
      )
        template(#empty)
          n-empty(
            description="Документы не найдены"
            size="large"
          )
            template(#icon)
              n-icon(:component="DocumentsOutline" size="48")

</template>

<script setup lang="ts">
import { ref, computed, onMounted, h, inject, onUnmounted } from 'vue'
import {
  NCard,
  NSpace,
  NText,
  NButton,
  NIcon,
  NInput,
  NSelect,
  NDataTable,
  NEmpty,
  NTag,
  NPopconfirm,
  NModal,
  type DataTableColumns,
  type PaginationProps,
  NTooltip
} from 'naive-ui'
import {
  SearchOutline,
  RefreshOutline,
  DocumentsOutline,
  LayersOutline,
  TrashOutline,
  CheckmarkCircleOutline,
  CloseCircleOutline,
  TimeOutline,
  CloudDoneOutline,
  HelpCircleOutline,
  AddOutline
} from '@vicons/ionicons5'
import { notify_service } from '@/services/notification_service'
import type { Document, DocumentStatus } from '@/types/document'
import { type ServiceStatus } from '@/types/service_status'
import { match } from '@globalart/oxide'
import { type Emitter } from 'strict-event-emitter'
import { type Events } from '@/services/emitter'
import DocumentAddForm from './DocumentAddForm.vue'
import useModelState from '@/composables/useModelState'
import useCollections from '@/composables/useCollections'
import useEmbeddings from '@/composables/useEmbeddings'
import useDocuments from '@/composables/useDocuments'

const emitter = inject<Emitter<Events>>('emitter') as Emitter<Events>;
const { getCollectionName, collectionOptions, getCollectionById} = useCollections()
const { documents, totalCount, loading: documentsLoading, loadDocuments, deleteDocument, addOrUpdateDocument, updateDocumentStatus } = useDocuments()
// Reactive state
const searchQuery = ref('')
const statusFilter = ref<DocumentStatus | null>(null)
const collectionFilter = ref<string | null>(null)
const currentPage = ref(1)
const pageSize = ref(20)
const show_add_modal = ref(false);
const {get_state} = useModelState()
const {embedding_document} = useEmbeddings();
//TODO не обновляется статус после окончания ембеддинга
// Опции для фильтра статуса
const statusOptions = [
  { label: 'Не загружен', value: 'NotLoaded' as DocumentStatus },
  { label: 'Загружен', value: 'Loaded' as DocumentStatus },
  { label: 'Эмбеддинги созданы', value: 'Embedded' as DocumentStatus },
  { label: 'Неизвестно', value: 'Unknown' as DocumentStatus }
]

// Pagination configuration
const pagination = computed<PaginationProps>(() => ({
  page: currentPage.value,
  pageSize: pageSize.value,
  pageCount: Math.ceil(totalCount.value / pageSize.value),
  showSizePicker: true,
  pageSizes: [10, 20, 30, 50],
  onChange: (page: number) => {
    currentPage.value = page
    handleLoadDocuments()
  },
  onUpdatePageSize: (size: number) => {
    pageSize.value = size
    currentPage.value = 1
    handleLoadDocuments()
  }
}))

// Фильтрация документов
const filteredDocuments = computed(() => {
  let result = documents.value

  // Фильтр по поисковому запросу
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(doc =>
      doc.document_title.toLowerCase().includes(query) ||
      doc.document_number.toLowerCase().includes(query)
    )
  }

  // Фильтр по статусу
  if (statusFilter.value) {
    result = result.filter(doc => doc.status === statusFilter.value)
  }

  // Фильтр по коллекции
  if (collectionFilter.value) {
    result = result.filter(doc => doc.collection_id === collectionFilter.value)
  }

  return result
})

// Форматирование даты
const formatDate = (dateTime: any): string => {
  if (!dateTime) return 'Не указана'
  const { day, month, year } = dateTime
  return `${String(day).padStart(2, '0')}.${String(month).padStart(2, '0')}.${year}`
}

// Получение метки статуса
const getStatusLabel = (status: DocumentStatus): string => {
  const statusMap: Record<DocumentStatus, string> = {
    'NotLoaded': 'Не загружен',
    'Loaded': 'Загружен',
    'Embedded': 'В поисковом индексе',
    'Unknown': 'Неизвестно'
  }
  return statusMap[status] || 'Неизвестно'
}

// Получение типа тега для статуса
const getStatusType = (status: DocumentStatus): 'default' | 'success' | 'warning' | 'error' | 'info' => {
  const typeMap: Record<DocumentStatus, 'default' | 'success' | 'warning' | 'error' | 'info'> = {
    'NotLoaded': 'default',
    'Loaded': 'info',
    'Embedded': 'success',
    'Unknown': 'warning'
  }
  return typeMap[status] || 'default'
}

// Получение иконки статуса
const getStatusIcon = (status: DocumentStatus) => {
  const iconMap: Record<DocumentStatus, any> = {
    'NotLoaded': TimeOutline,
    'Loaded': CloudDoneOutline,
    'Embedded': CheckmarkCircleOutline,
    'Unknown': HelpCircleOutline
  }
  return iconMap[status] || HelpCircleOutline
}

// Обработчик создания эмбеддингов
const handleEmbedding = async (document: Document) =>
{
  if (document.has_embeddings)
  {
    notify_service.info('Эмбеддинги уже созданы', `Для документа ${document.document_number} эмбеддинги уже существуют`)
    return
  }
  //TODO результат приходит сразу, нужно отслеживать статус документа и обновлять его в списке
  const success = await embedding_document(document.document_hash, document.collection_id)
  if (success)
  {
    notify_service.info("Процесс эмбеддинга запущен");
  }
}

// Обработчик удаления документа
const handleDelete = async (document: Document) => {
  await deleteDocument(document.document_hash)
}

// Загрузка документов
const handleLoadDocuments = async () =>
{
  try
  {
    const offset = (currentPage.value - 1) * pageSize.value
    await loadDocuments(offset, pageSize.value)
  }
  catch (error)
  {
    console.error('Error loading documents:', error)
    notify_service.error('Ошибка загрузки', 'Не удалось загрузить список документов')
  }
}

// Колонки таблицы
const columns = computed<DataTableColumns<Document>>(() => [
  {
    title: 'Номер',
    key: 'document_number',
    width: 150,
    ellipsis: {
      tooltip: true
    }
  },
  {
    title: 'Название',
    key: 'document_title',
    ellipsis: {
      tooltip: true
    }
  },
  {
    title: 'Коллекция',
    key: 'collection_id',
    width: 180,
    render: (row) => {
      return h(
        NTooltip,
        {
         
        },
        {
          trigger: () => h( 
            NTag,
            {
              type: 'info',
              bordered: false,
              size: 'small'
            },
            { default: () => getCollectionName(row.collection_id) }
          ),
          default: () =>  getCollectionById(row.collection_id)?.description || 'Нет описания'
        }
      )
    }
  },
  {
    title: 'Дата подписания',
    key: 'document_sign_date',
    width: 140,
    render: (row) => formatDate(row.document_sign_date)
  },
  {
    title: 'Статус',
    key: 'status',
    width: 180,
    render: (row) => {
      return h(
        NTag,
        {
          type: getStatusType(row.status),
          bordered: false
        },
        {
          default: () => getStatusLabel(row.status),
          icon: () => h(NIcon, { component: getStatusIcon(row.status) })
        }
      )
    }
  },
  {
    title: 'Чанки',
    key: 'chunks_count',
    width: 100,
    align: 'center'
  },
  {
    title: 'Действия',
    key: 'actions',
    width: 200,
    align: 'center',
    render: (row) => {
      return h(
        NSpace,
        { size: 8 },
        {
          default: () => [
            h(
              NButton,
              {
                size: 'small',
                type: row.has_embeddings ? 'success' : 'primary',
                disabled: row.has_embeddings || !get_state().value?.retriver,
                onClick: () => handleEmbedding(row)
              },
              {
                default: () => row.has_embeddings ? 'Готово' : 'Эмбеддинг',
                icon: () => h(NIcon, { component: LayersOutline })
              }
            ),
            h(
              NPopconfirm,
              {
                onPositiveClick: () => handleDelete(row)
              },
              {
                default: () => 'Вы уверены, что хотите удалить этот документ?',
                trigger: () => h(
                  NButton,
                  {
                    size: 'small',
                    type: 'error'
                  },
                  {
                    default: () => 'Удалить',
                    icon: () => h(NIcon, { component: TrashOutline })
                  }
                )
              }
            )
          ]
        }
      )
    }
  }
])

// Обработчик добавления нового документа
const handleDocumentAdded = (document: Document) =>
{
  const isNew = !documents.value.find(d => d.document_hash === document.document_hash)
  addOrUpdateDocument(document)
  if (isNew) {
    notify_service.success('Документ добавлен', 'Новый документ добавлен в список')
  } else {
    notify_service.info('Документ обновлен', 'Документ уже существовал в списке и был обновлен')
  }
  show_add_modal.value = false
}

// Загрузка документов при монтировании компонента
onMounted(() => {
  handleLoadDocuments()
  emitter.on('service_status', handleServiceStatusUpdate)
})

onUnmounted(() => {
  emitter.off('service_status', handleServiceStatusUpdate)
})

const handleServiceStatusUpdate = (status: ServiceStatus) =>
{
  match(status.status,
    [
      ['Complete', () => updateDocumentStatus(status.hash, 'Embedded', true)],
      ['Error', () => notify_service.error(status.message)],
      () => console.warn('service status:', status.status)
    ]
  )
  console.log('Received service status update:', status)
}

</script>

<style scoped>
.documents-list {
  max-width: 1400px;
  margin: 0 auto;
  padding: 24px;
}

:deep(.n-card) {
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
  border-radius: 12px;
}

:deep(.n-data-table) {
  margin-top: 16px;
}

:deep(.n-button) {
  font-weight: 500;
  transition: all 0.3s ease;
}

:deep(.n-button:hover:not(:disabled)) {
  transform: translateY(-2px);
}

:deep(.n-input) {
  border-radius: 8px;
}

:deep(.n-select) {
  border-radius: 8px;
}
</style>
