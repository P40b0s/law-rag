<template lang="pug">
n-card.queries(title="Поиск по базе знаний" :bordered="false")
  n-space(vertical :size="16")
    // Форма поиска
    n-form(
      ref="formRef"
      :model="formData"
      :rules="formRules"
    )
      n-form-item(path="query")
        n-input(
          v-model:value="formData.query"
          type="textarea"
          placeholder="Введите ваш вопрос..."
          :autosize="{ minRows: 3, maxRows: 6 }"
          clearable
          @keydown.ctrl.enter="handleSearch"
        )
          template(#suffix)
            n-icon(:component="SearchOutline" style="cursor: pointer" @click="handleSearch")

      // Настройки поиска
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
            :loading="isSearching"
            :disabled="!formData.query.trim()"
            @click="handleSearch"
          )
            template(#icon)
              n-icon(:component="SearchOutline")
            | Найти

          n-button(
            v-if="searchResults.length > 0"
            @click="handleClear"
          )
            | Очистить результаты

    // Результаты поиска
    n-spin(:show="isSearching")
      template(v-if="searchResults.length > 0")
        n-divider Найдено результатов: {{ searchResults.length }}

        .results-container
          n-list(hoverable clickable bordered)
            n-list-item(
              v-for="(result, index) in searchResults"
              :key="result.id"
            )
              template(#prefix)
                n-tag(
                  :bordered="false"
                  type="info"
                  size="small"
                  round
                ) {{ index + 1 }}

              n-thing(
                :title="`${result.document_title}`"
                :description="`${result.document_number} от ${result.document_sign_date}`"
              )
                template(#header-extra)
                  n-space
                    n-tag(
                      :bordered="false"
                      type="success"
                      size="small"
                    )
                      template(#icon)
                        n-icon(:component="TrendingUpOutline")
                      | Reranker: {{ (result.reranker_score * 100).toFixed(1) }}%

                    n-tag(
                      :bordered="false"
                      type="info"
                      size="small"
                    )
                      template(#icon)
                        n-icon(:component="StarOutline")
                      | Score: {{ (result.original_score * 100).toFixed(1) }}%

                template(#description)
                  n-space(vertical :size="8")
                    n-text(depth="3")
                      | {{ result.document_number }} от {{ result.document_sign_date }}

                    n-text(type="primary" style="font-size: 13px")
                      | Путь: {{ result.path }}

                    n-text(depth="2" style="font-size: 14px; line-height: 1.6")
                      | {{ result.text }}

                template(#action)
                  n-space
                    n-button(
                      text
                      tag="a"
                      :href="result.document_uri"
                      target="_blank"
                    )
                      template(#icon)
                        n-icon(:component="OpenOutline")
                      | Открыть документ

                    n-button(
                      text
                      @click="copyToClipboard(result.text)"
                    )
                      template(#icon)
                        n-icon(:component="CopyOutline")
                      | Копировать текст

      template(v-else-if="!isSearching && hasSearched")
        n-empty(
          description="Результатов не найдено"
          size="large"
        )
          template(#icon)
            n-icon(:component="DocumentTextOutline" size="48")
</template>

<script setup lang="ts">
import { ref } from 'vue'
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
  NList,
  NListItem,
  NThing,
  NTag,
  NText,
  NEmpty,
  type FormInst,
  type FormRules,
  useMessage
} from 'naive-ui'
import {
  SearchOutline,
  TrendingUpOutline,
  StarOutline,
  OpenOutline,
  CopyOutline,
  DocumentTextOutline
} from '@vicons/ionicons5'
import { http_service } from '@/services/http_service/http_service'
import type { QdrantContext } from '@/types/query'
import {notify_service} from '@/services/notification_service';

// Reactive state
const formRef = ref<FormInst | null>(null)
const isSearching = ref(false)
const hasSearched = ref(false)
const searchResults = ref<QdrantContext[]>([])
const message = notify_service;

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

// Methods
const handleSearch = async () => {
  if (!formRef.value) return

  try {
    await formRef.value.validate()

    if (!formData.value.query.trim()) {
      message.warning('Введите запрос для поиска')
      return
    }

    isSearching.value = true
    hasSearched.value = true

    const results = await http_service.query_service.search_results(
      formData.value.query,
      formData.value.limit,
      formData.value.reranker_limit
    )

    if (results) {
      searchResults.value = results
    } else {
      searchResults.value = []
    }
  } catch (error) {
    console.error('Error searching:', error)
    message.error('Произошла ошибка при поиске')
    searchResults.value = []
  } finally {
    isSearching.value = false
  }
}

const handleClear = () => {
  searchResults.value = []
  hasSearched.value = false
}

const copyToClipboard = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    message.success('Текст скопирован в буфер обмена')
  } catch (error) {
    console.error('Error copying to clipboard:', error)
    message.error('Не удалось скопировать текст')
  }
}
</script>

<style scoped>
:deep(.n-card) {
  margin-bottom: 24px;
  width: 100%;
  max-width: 100%;
}

.results-container {
  overflow-y: auto;
  max-height: 300px;
}
.queries
{
  max-height: calc(100vh - 100px);
  width: 100vw;
}

:deep(.n-thing-header__title) {
  font-size: 16px;
  font-weight: 600;
}

:deep(.n-list-item) {
  padding: 16px;
}

:deep(.n-thing) {
  width: 100%;
}
</style>
