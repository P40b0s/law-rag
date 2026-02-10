<template lang="pug">
.status-bar
  .progress-bar(:class="progressBarClass" :style="{ width: progressWidth }")
  n-space(:size="16" align="center" justify="space-between")
    n-space(:size="12" align="center")
      // Статус подключения к API
      n-tag(
        :type="apiStatus.connected ? 'success' : 'error'"
        :bordered="false"
        size="small"
        round
      )
        template(#icon)
          n-icon(:component="apiStatus.connected ? WifiOutline : WifiOffOutline")
        | {{ apiStatus.connected ? 'Подключено' : 'Нет подключения' }}

      // Статус операций от сервиса
      n-tag(
        v-if="currentStatus"
        :type="getStatusType(currentStatus.status)"
        :bordered="false"
        size="small"
        round
      )
        template(#icon)
          n-icon(:component="getStatusIcon(currentStatus.status)")
        | {{ getStatusLabel(currentStatus.status) }}
        span(v-if="hasProgress" style="margin-left: 8px")
          | ({{ currentStatus.current_chunk }}/{{ currentStatus.overall_chunks }})

      // Сообщение от сервиса
      n-tag(
        v-if="currentStatus?.message"
        type="info"
        :bordered="false"
        size="small"
        round
      )
        template(#icon)
          n-icon(:component="InformationCircleOutline")
        n-ellipsis(:style="{ maxWidth: '300px' }")
          | {{ currentStatus.message }}

    // Информация о последнем обновлении
    n-space(:size="8" align="center")
      n-text(depth="3" :style="{ fontSize: '12px' }")
        | {{ lastUpdateTime }}

      n-button(
        v-if="currentStatus && currentStatus.status !== 'Error'"
        text
        size="tiny"
        @click="handleDismiss"
      )
        template(#icon)
          n-icon(:component="CloseOutline")

</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, inject } from 'vue'
import {
  NSpace,
  NTag,
  NIcon,
  NText,
  NButton,
  NEllipsis
} from 'naive-ui'
import {
  WifiOutline,
  HeartDislike as WifiOffOutline,
  LayersOutline,
  RefreshOutline,
  SparklesOutline,
  SearchOutline,
  DocumentTextOutline,
  InformationCircleOutline,
  CloseCircleOutline,
  CloseOutline,
  CheckmarkCircleOutline,
  ChevronDownCircleOutline
} from '@vicons/ionicons5'
import { http_sevice } from '@/services/http_service/http_service'
import { Emitter } from 'strict-event-emitter'
import { type Events } from '@/services/emitter'
import { type ServiceStatus } from '@/types/service_status'

// Emitter для получения событий от сервиса
const emitter = inject<Emitter<Events>>('emitter') as Emitter<Events>;
const currentStatus = ref<ServiceStatus | null>(null);
const lastUpdate = ref(Date.now())

// Reactive state
const apiStatus = ref({
  connected: true,
  lastCheck: Date.now()
})

// Computed
const hasProgress = computed(() => 
{
  return currentStatus.value?.current_chunk !== undefined &&
         currentStatus.value?.overall_chunks !== undefined
})

const progressWidth = computed(() => 
{
  if (!currentStatus.value || !hasProgress.value) 
  {
    return '0%'
  }

  const current = currentStatus.value.current_chunk || 0
  const total = currentStatus.value.overall_chunks || 1
  const percentage = Math.min((current / total) * 100, 100)

  return `${percentage}%`
})

const progressBarClass = computed(() => 
{
  if (!currentStatus.value) 
  {
    return 'progress-idle'
  }

  if (currentStatus.value.status === 'Error') 
  {
    return 'progress-error'
  }

  if (['Embedding', 'Chunking', 'Reranking', 'Generation'].includes(currentStatus.value.status)) 
  {
    return 'progress-active'
  }

  if (currentStatus.value.status === 'Complete') 
  {
    return 'progress-complete'
  }

  return 'progress-idle'
})

const lastUpdateTime = computed(() => {
  if (!currentStatus.value) return 'Готово'

  const now = Date.now()
  const diff = now - lastUpdate.value

  if (diff < 5000) {
    return 'сейчас'
  } else if (diff < 60000) {
    const seconds = Math.floor(diff / 1000)
    return `${seconds} сек. назад`
  } else if (diff < 3600000) {
    const minutes = Math.floor(diff / 60000)
    return `${minutes} мин. назад`
  } else {
    const hours = Math.floor(diff / 3600000)
    return `${hours} ч. назад`
  }
})

// Интервал для обновления времени
let timeUpdateInterval: number | null = null

// Получение типа тега для статуса
const getStatusType = (status: string): 'default' | 'success' | 'warning' | 'error' | 'info' => {
  const typeMap: Record<string, 'default' | 'success' | 'warning' | 'error' | 'info'> = {
    'Embedding': 'warning',
    'Reranking': 'info',
    'Generation': 'success',
    'Chunking': 'info',
    'Message': 'default',
    'Error': 'error'
  }
  return typeMap[status] || 'default'
}

// Получение метки статуса
const getStatusLabel = (status: string): string => {
  const labelMap: Record<string, string> = {
    'Embedding': 'Создание эмбеддингов',
    'Reranking': 'Ранжирование',
    'Generation': 'Генерация ответа',
    'Chunking': 'Разбиение на чанки',
    'Complete': 'Завершено',
    'Message': 'Сообщение',
    'Error': 'Ошибка'
  }
  return labelMap[status] || status
}

// Получение иконки статуса
const getStatusIcon = (status: string) => {
  const iconMap: Record<string, any> = {
    'Embedding': LayersOutline,
    'Reranking': SearchOutline,
    'Generation': SparklesOutline,
    'Chunking': DocumentTextOutline,
    'Message': InformationCircleOutline,
    'Complete': ChevronDownCircleOutline,
    'Error': CloseCircleOutline
  }
  return iconMap[status] || InformationCircleOutline
}

// Проверка статуса API (для проверки подключения)
const checkApiStatus = async () => {
  try {
    const result = await http_sevice.documents_service.health_check()
    apiStatus.value.connected = result !== undefined
    apiStatus.value.lastCheck = Date.now()
  } catch (error) {
    apiStatus.value.connected = false
    apiStatus.value.lastCheck = Date.now()
  }
}

// Обработчик обновления статуса сервиса
const handleServiceStatusUpdate = (status: ServiceStatus) => {
  currentStatus.value = status
  lastUpdate.value = Date.now()

  // Обновляем статус подключения, если получили событие
  apiStatus.value.connected = true
  apiStatus.value.lastCheck = Date.now()
}

// Закрыть текущее сообщение о статусе
const handleDismiss = () => {
  currentStatus.value = null
}

// Lifecycle hooks
onMounted(() => {
  // Первоначальная проверка подключения
  checkApiStatus()

  // Подписываемся на события статуса
  emitter.on('service_status', handleServiceStatusUpdate)

  // Обновляем отображение времени каждые 5 секунд
  timeUpdateInterval = window.setInterval(() => {
    // Принудительно обновляем computed значение
    lastUpdate.value = lastUpdate.value
  }, 5000)
})

onUnmounted(() => {
  // Отписываемся от событий
  emitter.off('service_status', handleServiceStatusUpdate)

  if (timeUpdateInterval) {
    clearInterval(timeUpdateInterval)
  }
})
</script>

<style scoped>
.status-bar {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  background: linear-gradient(135deg, #383f49 0%, #212e3a 100%);
  padding: 8px 24px;
  z-index: 1000;
  backdrop-filter: blur(10px);
  box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.05);
  animation: slideUp 0.3s ease-out;
  min-height: 40px;
}

/* Прогресс-бар */
.progress-bar {
  position: absolute;
  top: 0;
  left: 0;
  height: 3px;
  transition: width 0.3s ease-out, background-color 0.3s ease;
  z-index: 1001;
}

/* Состояния прогресс-бара */
.progress-idle {
  background-color: #427bc0;
  width: 0%;
}

.progress-active {
  background: linear-gradient(90deg, #f0a020 0%, #f7ba2a 50%, #f0a020 100%);
  background-size: 200% 100%;
  animation: shimmer 2s linear infinite;
  box-shadow: 0 0 10px rgba(240, 160, 32, 0.5);
}

.progress-error {
  background-color: #d03050;
  box-shadow: 0 0 10px rgba(208, 48, 80, 0.5);
}

.progress-complete {
  background-color: #18a058;
  box-shadow: 0 0 10px rgba(24, 160, 88, 0.5);
  transition: width 0.5s ease-out, opacity 1s ease 0.5s;
}

/* Анимация мерцания для активного прогресса */
@keyframes shimmer {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}

:deep(.n-tag) {
  font-weight: 500;
  padding: 4px 12px;
  transition: all 0.3s ease;
  animation: fadeIn 0.3s ease-out;
}

:deep(.n-button) {
  transition: all 0.3s ease;
  opacity: 0.7;
}

:deep(.n-button:hover) {
  opacity: 1;
  transform: scale(1.1);
}

:deep(.n-ellipsis) {
  display: inline-block;
}

/* Анимации */
@keyframes slideUp {
  from {
    transform: translateY(100%);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: scale(0.9);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

/* Пульсация для активных операций */
:deep(.n-tag[type="warning"]),
:deep(.n-tag[type="info"]) {
  animation: fadeIn 0.3s ease-out, pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.8;
  }
}

/* Адаптивность для мобильных устройств */
@media (max-width: 768px) {
  .status-bar {
    padding: 6px 12px;
  }

  :deep(.n-space) {
    flex-wrap: wrap;
  }

  :deep(.n-tag) {
    font-size: 11px;
    padding: 3px 8px;
  }

  :deep(.n-ellipsis) {
    max-width: 150px !important;
  }
}
</style>
