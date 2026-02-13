<template lang="pug">
n-card.generation(title="Генерация ответа" :bordered="false")
  n-space(vertical :size="16")
    // Форма запроса
    n-form(
      ref="formRef"
      :model="formData"
      :rules="formRules"
    )
      n-form-item(path="query")
        n-input(
          v-model:value="formData.query"
          type="textarea"
          placeholder="Введите ваш вопрос для генерации ответа..."
          :autosize="{ minRows: 3, maxRows: 6 }"
          clearable
          @keydown.ctrl.enter="handleGenerate"
        )
          template(#suffix)
            n-icon(:component="SparklesOutline" style="cursor: pointer" @click="handleGenerate")

      // Настройки генерации
      n-collapse(arrow-placement="right")
        n-collapse-item(title="Расширенные настройки" name="advanced")
          n-space(vertical :size="12")
            n-form-item(label="Лимит результатов" path="limit")
              n-slider(
                v-model:value="formData.limit"
                :min="5"
                :max="50"
                :step="5"
                :marks="{ 5: '5', 25: '25', 50: '50' }"
              )

            n-form-item(label="Лимит reranker" path="reranker_limit")
              n-slider(
                v-model:value="formData.reranker_limit"
                :min="3"
                :max="20"
                :step="1"
                :marks="{ 3: '3', 10: '10', 20: '20' }"
              )

      n-form-item
        n-space
          n-button(
            type="primary"
            :loading="isGenerating"
            :disabled="!formData.query.trim()"
            @click="handleGenerate"
          )
            template(#icon)
              n-icon(:component="SparklesOutline")
            | Сгенерировать ответ

          n-button(
            v-if="generatedText"
            @click="handleClear"
          )
            | Очистить

    // Результат генерации
    n-spin(:show="isGenerating")
      template(v-if="generatedText")
        n-divider Сгенерированный ответ

        .answer-container
          .markdown-content(v-html="renderedMarkdown")

        n-space(:size="8" style="margin-top: 12px")
          n-button(
            secondary
            @click="copyToClipboard(generatedText)"
          )
            template(#icon)
              n-icon(:component="CopyOutline")
            | Копировать текст

      template(v-else-if="!isGenerating && hasGenerated")
        n-empty(
          description="Не удалось сгенерировать ответ"
          size="large"
        )
          template(#icon)
            n-icon(:component="DocumentTextOutline" size="48")
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, inject, computed } from 'vue'
import {
  NCard,
  NForm,
  NFormItem,
  NInput,
  NButton,
  NSpace,
  NIcon,
  NSlider,
  NCollapse,
  NCollapseItem,
  NSpin,
  NDivider,
  NEmpty,
  type FormInst,
  type FormRules
} from 'naive-ui'
import {
  SparklesOutline,
  CopyOutline,
  DocumentTextOutline
} from '@vicons/ionicons5'
import { http_service } from '@/services/http_service/http_service'
import { notify_service } from '@/services/notification_service'
import { Emitter } from 'strict-event-emitter'
import { type Events } from '@/services/emitter'
import { type ServiceStatus } from '@/types/service_status'
import { marked } from 'marked'
import DOMPurify from 'dompurify'

// Emitter для получения событий от сервиса
const emitter = inject<Emitter<Events>>('emitter') as Emitter<Events>

// Reactive state
const formRef = ref<FormInst | null>(null)
const isGenerating = ref(false)
const hasGenerated = ref(false)
const generatedText = ref('')

const formData = ref({
  query: '',
  limit: 10,
  reranker_limit: 5
})

// Form rules
const formRules: FormRules = {
  query: {
    required: true,
    message: 'Введите ваш вопрос',
    trigger: ['input', 'blur']
  },
  limit: {
    type: 'number',
    required: true,
    message: 'Укажите лимит результатов',
    trigger: 'change'
  },
  reranker_limit: {
    type: 'number',
    required: true,
    message: 'Укажите лимит reranker',
    trigger: 'change'
  }
}

// Computed для рендеринга markdown
const renderedMarkdown = computed(() => {
  if (!generatedText.value) return ''

  // Парсим markdown в HTML
  const rawHtml = marked.parse(generatedText.value, { async: false }) as string

  // Санитизируем HTML для безопасности
  return DOMPurify.sanitize(rawHtml, {
    ALLOWED_TAGS: [
      'p', 'br', 'strong', 'em', 'u', 's', 'code', 'pre',
      'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
      'ul', 'ol', 'li',
      'blockquote',
      'a', 'img',
      'table', 'thead', 'tbody', 'tr', 'th', 'td',
      'hr', 'div', 'span'
    ],
    ALLOWED_ATTR: ['href', 'src', 'alt', 'title', 'class', 'target', 'rel']
  })
})

// Обработчик события генерации от SSE
const handleGenerationEvent = (status: ServiceStatus) => {
  // Фильтруем только события генерации
  if (status.status === 'Generation') {
    // Добавляем полученный текст к существующему
    generatedText.value += status.message
  } else if (status.status === 'Complete') {
    // Генерация завершена
    isGenerating.value = false
  } else if (status.status === 'Error') {
    // Ошибка при генерации
    isGenerating.value = false
    notify_service.error('Ошибка генерации', status.message)
  }
}

// Methods
const handleGenerate = async () => {
  if (!formRef.value) return

  try {
    await formRef.value.validate()

    if (!formData.value.query.trim()) {
      notify_service.warning('Введите запрос для генерации')
      return
    }

    // Очищаем предыдущий результат
    generatedText.value = ''
    isGenerating.value = true
    hasGenerated.value = true

    // Отправляем запрос на генерацию
    const success = await http_service.query_service.generation_request(
      formData.value.query,
      formData.value.limit,
      formData.value.reranker_limit
    )

    if (!success) {
      isGenerating.value = false
      hasGenerated.value = true
      notify_service.error('Не удалось запустить генерацию')
    }
    // SSE события будут приходить через emitter и обрабатываться в handleGenerationEvent
  } catch (error) {
    console.error('Error generating:', error)
    notify_service.error('Произошла ошибка при генерации')
    isGenerating.value = false
    hasGenerated.value = true
  }
}

const handleClear = () => {
  generatedText.value = ''
  hasGenerated.value = false
}

const copyToClipboard = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    notify_service.success('Текст скопирован в буфер обмена')
  } catch (error) {
    console.error('Error copying to clipboard:', error)
    notify_service.error('Не удалось скопировать текст')
  }
}

// Lifecycle hooks
onMounted(() => {
  // Подписываемся на события генерации через SSE
  emitter.on('service_status', handleGenerationEvent)
})

onUnmounted(() => {
  // Отписываемся от событий
  emitter.off('service_status', handleGenerationEvent)
})
</script>

<style scoped>
:deep(.n-card) {
  margin-bottom: 24px;
  width: 100%;
  max-width: 100%;
}

.answer-container {
  width: inherit;
  max-height: 600px;
  overflow-y: auto;
  padding: 16px;
  background-color: #f5f5f5;
  border-radius: 8px;
  border: 1px solid #e0e0e0;
}

.generation {
  max-height: calc(100vh - 100px);
  width: 100vw;
}

:deep(.n-input) {
  width: 100%;
}

:deep(.n-input__textarea-el) {
  font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
  font-size: 14px;
  line-height: 1.6;
}

/* Markdown стили */
.markdown-content {
  font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
  font-size: 14px;
  line-height: 1.8;
  color: #333;
  word-wrap: break-word;
}

.markdown-content :deep(h1),
.markdown-content :deep(h2),
.markdown-content :deep(h3),
.markdown-content :deep(h4),
.markdown-content :deep(h5),
.markdown-content :deep(h6) {
  margin-top: 24px;
  margin-bottom: 16px;
  font-weight: 600;
  line-height: 1.25;
  color: #1a1a1a;
}

.markdown-content :deep(h1) {
  font-size: 2em;
  border-bottom: 2px solid #e0e0e0;
  padding-bottom: 8px;
}

.markdown-content :deep(h2) {
  font-size: 1.5em;
  border-bottom: 1px solid #e0e0e0;
  padding-bottom: 6px;
}

.markdown-content :deep(h3) {
  font-size: 1.25em;
}

.markdown-content :deep(h4) {
  font-size: 1em;
}

.markdown-content :deep(p) {
  margin-bottom: 16px;
}

.markdown-content :deep(ul),
.markdown-content :deep(ol) {
  margin-bottom: 16px;
  padding-left: 2em;
}

.markdown-content :deep(li) {
  margin-bottom: 8px;
}

.markdown-content :deep(code) {
  background-color: #f0f0f0;
  border-radius: 4px;
  padding: 2px 6px;
  font-family: 'Courier New', Courier, monospace;
  font-size: 0.9em;
  color: #d63384;
}

.markdown-content :deep(pre) {
  background-color: #2d2d2d;
  color: #f8f8f2;
  border-radius: 6px;
  padding: 16px;
  overflow-x: auto;
  margin-bottom: 16px;
}

.markdown-content :deep(pre code) {
  background-color: transparent;
  color: inherit;
  padding: 0;
}

.markdown-content :deep(blockquote) {
  border-left: 4px solid #d0d0d0;
  padding-left: 16px;
  margin: 16px 0;
  color: #666;
  font-style: italic;
}

.markdown-content :deep(a) {
  color: #2080f0;
  text-decoration: none;
}

.markdown-content :deep(a:hover) {
  text-decoration: underline;
}

.markdown-content :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin-bottom: 16px;
}

.markdown-content :deep(th),
.markdown-content :deep(td) {
  border: 1px solid #ddd;
  padding: 8px 12px;
  text-align: left;
}

.markdown-content :deep(th) {
  background-color: #f0f0f0;
  font-weight: 600;
}

.markdown-content :deep(tr:nth-child(even)) {
  background-color: #fafafa;
}

.markdown-content :deep(hr) {
  border: none;
  border-top: 1px solid #e0e0e0;
  margin: 24px 0;
}

.markdown-content :deep(img) {
  max-width: 100%;
  height: auto;
  border-radius: 4px;
}

.markdown-content :deep(strong) {
  font-weight: 600;
}

.markdown-content :deep(em) {
  font-style: italic;
}
</style>
