<template>
  <div class="documents-manager">
    <!-- Диалог процесса обработки -->
    <n-modal
      v-model:show="showProcessDialog"
      preset="dialog"
      title="Процесс обработки документов"
      :mask-closable="false"
    >
      <div class="process-dialog">
        <n-space vertical>
          <n-alert
            v-if="processingError"
            title="Ошибка обработки"
            type="error"
            closable
            @close="processingError = null"
          >
            {{ processingError }}
          </n-alert>
          
          <div v-if="currentProcess">
            <n-space vertical>
              <n-progress
                type="line"
                :percentage="progressPercentage"
                :status="progressStatus"
                :height="24"
              />
              <n-text type="info" depth="3">
                Обрабатывается: {{ currentProcess.hash }}
              </n-text>
              <n-text>
                Статус: {{ currentProcess.status }}
              </n-text>
              <n-text>
                Прогресс: {{ currentProcess.current_chunk }} / {{ currentProcess.overall_chunks }}
              </n-text>
            </n-space>
          </div>
          
          <n-empty
            v-else
            description="Нет активных процессов обработки"
          />
        </n-space>
      </div>
      
      <template #action>
        <n-button @click="stopProcessing" :disabled="!currentProcess">
          Остановить
        </n-button>
        <n-button type="primary" @click="showProcessDialog = false">
          Закрыть
        </n-button>
      </template>
    </n-modal>

    <!-- Диалог добавления документа -->
    <n-modal
      v-model:show="showAddDialog"
      preset="dialog"
      title="Добавить документ"
    >
      <n-form
        ref="addFormRef"
        :model="newDocument"
        :rules="addFormRules"
        label-placement="left"
        label-width="auto"
        require-mark-placement="right-hanging"
      >
        <n-form-item label="URI документа" path="document_uri">
          <n-input
            v-model:value="newDocument.document_uri"
            placeholder="Введите URI документа"
          />
        </n-form-item>
        
        <n-form-item label="Заголовок" path="document_title">
          <n-input
            v-model:value="newDocument.document_title"
            placeholder="Введите заголовок документа"
          />
        </n-form-item>
        
        <n-form-item label="Номер документа" path="document_number">
          <n-input
            v-model:value="newDocument.document_number"
            placeholder="Введите номер документа"
          />
        </n-form-item>
        
        <n-form-item label="Дата подписания" path="document_sign_date">
          <n-date-picker
            v-model:value="newDocument.document_sign_date"
            type="date"
            placeholder="Выберите дату"
          />
        </n-form-item>
      </n-form>
      
      <template #action>
        <n-button @click="showAddDialog = false">
          Отмена
        </n-button>
        <n-button type="primary" @click="addDocument" :loading="addingDocument">
          Добавить
        </n-button>
      </template>
    </n-modal>

    <!-- Панель управления -->
    <div class="control-panel">
      <n-space>
        <n-button type="primary" @click="showAddDialog = true">
          <template #icon>
            <n-icon><PlusIcon /></n-icon>
          </template>
          Добавить документ
        </n-button>
        
        <n-button @click="showProcessDialog = true" :disabled="!currentProcess">
          <template #icon>
            <n-icon><ProcessIcon /></n-icon>
          </template>
          Показать процесс
        </n-button>
        
        <n-button type="info" @click="startProcessing" :loading="startingProcessing">
          <template #icon>
            <n-icon><PlayIcon /></n-icon>
          </template>
          Запустить обработку
        </n-button>
        
        <n-button type="warning" @click="refreshDocuments">
          <template #icon>
            <n-icon><RefreshIcon /></n-icon>
          </template>
          Обновить
        </n-button>
      </n-space>
    </div>

    <!-- Таблица документов -->
    <n-card title="Документы">
      <n-data-table
        :columns="columns"
        :data="documents"
        :loading="loading"
        :row-key="(row: Document) => row.document_hash"
        :pagination="pagination"
      />
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, h } from 'vue'
import {
  NDataTable,
  NButton,
  NModal,
  NForm,
  NFormItem,
  NInput,
  NDatePicker,
  NSpace,
  NCard,
  NProgress,
  NText,
  NEmpty,
  NAlert,
  NIcon,
  useMessage,
  type DataTableColumns,
  type FormRules,
  type FormInst
} from 'naive-ui'
import { Add, Play, Refresh, Eye, Trash, Cog } from '@vicons/fa'

// Иконки
const PlusIcon = Add
const PlayIcon = Play
const RefreshIcon = Refresh
const ProcessIcon = Cog

// Типы
interface Document 
{
  document_uri: string
  document_hash: string
  document_title: string
  document_number: string
  document_sign_date: number // timestamp
  has_embeddings: boolean
  chunks_count: number
  chunks: Chunk[]
  status: number
}

interface Chunk {
  id: string
  content: string
  // Добавьте другие поля по необходимости
}

interface ChunkProcess 
{
  hash: string
  current_chunk: number
  overall_chunks: number
  status: string
  is_error: boolean
}

// Реактивные переменные
const documents = ref<Document[]>([])
const loading = ref(false)
const showAddDialog = ref(false)
const showProcessDialog = ref(false)
const currentProcess = ref<ChunkProcess | null>(null)
const processingError = ref<string | null>(null)
const addingDocument = ref(false)
const startingProcessing = ref(false)
const message = useMessage()

// Форма добавления
const addFormRef = ref<FormInst | null>(null)
const newDocument = ref({
  document_uri: '',
  document_title: '',
  document_number: '',
  document_sign_date: Date.now()
})

// Правила валидации
const addFormRules: FormRules = {
  document_uri: {
    required: true,
    message: 'URI документа обязателен',
    trigger: ['input', 'blur']
  },
  document_title: {
    required: true,
    message: 'Заголовок обязателен',
    trigger: ['input', 'blur']
  },
  document_number: {
    required: true,
    message: 'Номер документа обязателен',
    trigger: ['input', 'blur']
  }
}

// Пагинация
const pagination = {
  pageSize: 10
}

// Колонки таблицы
const columns: DataTableColumns<Document> = [
  {
    title: 'Заголовок',
    key: 'document_title',
    width: 200,
    ellipsis: {
      tooltip: true
    }
  },
  {
    title: 'Номер',
    key: 'document_number',
    width: 120
  },
  {
    title: 'Дата подписания',
    key: 'document_sign_date',
    width: 150,
    render: (row) => {
      return format(new Date(row.document_sign_date), 'dd.MM.yyyy')
    }
  },
  {
    title: 'Чанков',
    key: 'chunks_count',
    width: 100,
    align: 'center'
  },
  {
    title: 'Эмбеддинги',
    key: 'has_embeddings',
    width: 120,
    render: (row) => {
      return row.has_embeddings ? 'Да' : 'Нет'
    }
  },
  {
    title: 'Статус',
    key: 'status',
    width: 100,
    render: (row) => {
      const statusMap: Record<number, string> = {
        0: 'Новый',
        1: 'Обрабатывается',
        2: 'Завершен',
        3: 'Ошибка'
      }
      return statusMap[row.status] || 'Неизвестно'
    }
  },
  {
    title: 'Действия',
    key: 'actions',
    width: 200,
    render: (row) => {
      return h(NSpace, {}, {
        default: () => [
          h(
            NButton,
            {
              size: 'small',
              type: 'info',
              onClick: () => viewDocument(row)
            },
            {
              icon: () => h(NIcon, null, { default: () => h(Eye) }),
              default: 'Просмотр'
            }
          ),
          h(
            NButton,
            {
              size: 'small',
              type: 'error',
              onClick: () => deleteDocument(row.document_hash)
            },
            {
              icon: () => h(NIcon, null, { default: () => h(Trash) }),
              default: 'Удалить'
            }
          )
        ]
      })
    }
  }
]

// Вычисляемые свойства
const progressPercentage = computed(() => {
  if (!currentProcess.value) return 0
  const { current_chunk, overall_chunks } = currentProcess.value
  return overall_chunks > 0 ? Math.round((current_chunk / overall_chunks) * 100) : 0
})

const progressStatus = computed(() => {
  if (!currentProcess.value) return 'default'
  return currentProcess.value.is_error ? 'error' : 'info'
})

// Методы
const fetchDocuments = async () => {
  loading.value = true
  try {
    const response = await axios.get('/api/documents')
    documents.value = response.data
  } catch (error) {
    message.error('Ошибка загрузки документов')
    console.error('Error fetching documents:', error)
  } finally {
    loading.value = false
  }
}

const fetchProcessStatus = async () => {
  try {
    const response = await axios.get('/api/process/status')
    currentProcess.value = response.data
    
    if (currentProcess.value?.is_error) {
      processingError.value = currentProcess.value.status
    }
  } catch (error) {
    console.error('Error fetching process status:', error)
  }
}

const addDocument = async () => {
  if (!addFormRef.value) return
  
  try {
    await addFormRef.value.validate()
    addingDocument.value = true
    
    const response = await axios.post('/api/documents', newDocument.value)
    
    if (response.data.success) {
      message.success('Документ добавлен')
      showAddDialog.value = false
      resetNewDocument()
      fetchDocuments()
    }
  } catch (error) {
    message.error('Ошибка добавления документа')
    console.error('Error adding document:', error)
  } finally {
    addingDocument.value = false
  }
}

const deleteDocument = async (hash: string) => {
  try {
    const response = await axios.delete(`/api/documents/${hash}`)
    
    if (response.data.success) {
      message.success('Документ удален')
      fetchDocuments()
    }
  } catch (error) {
    message.error('Ошибка удаления документа')
    console.error('Error deleting document:', error)
  }
}

const startProcessing = async () => {
  startingProcessing.value = true
  try {
    const response = await axios.post('/api/process/start')
    
    if (response.data.success) {
      message.success('Обработка запущена')
      showProcessDialog.value = true
      fetchProcessStatus()
    }
  } catch (error) {
    message.error('Ошибка запуска обработки')
    console.error('Error starting process:', error)
  } finally {
    startingProcessing.value = false
  }
}

const stopProcessing = async () => {
  try {
    const response = await axios.post('/api/process/stop')
    
    if (response.data.success) {
      message.success('Обработка остановлена')
      currentProcess.value = null
    }
  } catch (error) {
    message.error('Ошибка остановки обработки')
    console.error('Error stopping process:', error)
  }
}

const refreshDocuments = () => {
  fetchDocuments()
  if (showProcessDialog.value) {
    fetchProcessStatus()
  }
}

const viewDocument = (doc: Document) => {
  message.info(`Просмотр документа: ${doc.document_title}`)
  // Здесь можно реализовать логику просмотра документа
}

const resetNewDocument = () => {
  newDocument.value = {
    document_uri: '',
    document_title: '',
    document_number: '',
    document_sign_date: Date.now()
  }
}

// Хуки
onMounted(() => {
  fetchDocuments()
  
  // Опционально: обновление статуса процесса каждые 5 секунд
  setInterval(fetchProcessStatus, 5000)
})
</script>

<style scoped>
.documents-manager {
  padding: 20px;
}

.control-panel {
  margin-bottom: 20px;
}

.process-dialog {
  min-height: 150px;
  padding: 10px 0;
}

.n-button {
  margin-right: 8px;
}
</style>