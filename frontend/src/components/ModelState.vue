<template lang="pug">
.models-container(v-if="state")
  // Ретривер модель
  n-popover(
    trigger="hover"
    :show-arrow="true"
    placement="bottom"
    width="320"
  )
    template(#trigger)
      n-tag(
        :type="state.retriver ? 'success' : 'default'"
        :bordered="false"
        size="medium"
        round
        class="model-tag"
      )
        template(#icon)
          n-icon(:component="SearchCircleOutline" size="18")
        | Ретривер

    .popover-content
      .popover-header
        n-icon(:component="SearchCircleOutline" size="24" :color="state.retriver ? '#18a058' : '#909399'")
        .popover-title
          .model-name Ретривер
          .model-description Эмбеддинг модель для поиска

      .popover-divider

      .popover-control
        .control-label Состояние:
        n-switch(
          :value="!!state.retriver"
          @update:value="toggleModel('retriver')"
          :loading="loadingStates.retriver"
          :disabled="loadingStates.generator"
        )
          template(#checked) Загружен
          template(#unchecked) Выгружен

      .popover-info(v-if="state.retriver")
        .info-item
          n-icon(:component="ServerOutline" size="16")
          span {{ state.retriver.retriver_model_name }}
        .info-item
          n-icon(:component="ServerOutline" size="16")
          span {{ state.retriver.reranker_model_name }}

  // Генератор модель
  n-popover(
    trigger="hover"
    :show-arrow="true"
    placement="bottom"
    width="320"
  )
    template(#trigger)
      n-tag(
        :type="state.generator ? 'success' : 'default'"
        :bordered="false"
        size="medium"
        round
        class="model-tag"
      )
        template(#icon)
          n-icon(:component="SparklesOutline" size="18")
        | Генератор

    .popover-content
      .popover-header
        n-icon(:component="SparklesOutline" size="24" :color="state.generator ? '#18a058' : '#909399'")
        .popover-title
          .model-name Генератор
          .model-description LLM для генерации ответов

      .popover-divider

      .popover-control
        .control-label Состояние:
        n-switch(
          :value="!!state.generator"
          @update:value="toggleModel('generator')"
          :loading="loadingStates.generator"
          :disabled="loadingStates.retriver"
        )
          template(#checked) Загружен
          template(#unchecked) Выгружен

      .popover-info(v-if="state.generator")
        .info-item
          n-icon(:component="ServerOutline" size="16")
          span {{ state.generator.model_name }} ({{ formatBytes(state.generator.model_size) }})
        .info-item
          n-icon(:component="ServerOutline" size="16")
          span Температура: {{ state.generator.temperature }}

      .popover-section(v-if="state.generator?.system_prompt")
        .section-title
          n-icon(:component="DocumentTextOutline" size="16")
          span Системный промпт
        .system-prompt {{ state.generator.system_prompt }}

  // Кнопка управления всеми моделями
  n-popover(
    trigger="hover"
    :show-arrow="true"
    placement="bottom"
    width="280"
  )
    template(#trigger)
      n-tag(
        :type="getOverallStatusType()"
        :bordered="false"
        size="medium"
        round
        class="model-tag status-tag"
      )
        template(#icon)
          n-icon(:component="LayersOutline" size="18")
        | {{ loadedCount }}/2

    .popover-content
      .popover-header
        n-icon(:component="LayersOutline" size="24" :color="iconColor")
        .popover-title
          .model-name Модели
          .model-description {{ loadedCount }} из 2 загружено

      .popover-divider

      .popover-actions
        n-button(
          type="primary"
          @click="toggleAllModels"
          :loading="isLoadingAny"
          :disabled="isLoadingAny"
          block
          secondary
        )
          template(#icon)
            n-icon(:component="areAllModelsLoaded ? PowerOutline : PlayCircleOutline")
          | {{ areAllModelsLoaded ? 'Выгрузить все' : 'Загрузить все' }}
</template>

<script lang="ts" setup>
import { computed, ref, reactive } from 'vue'
import { NPopover, NIcon, NTag, NSwitch, NButton} from 'naive-ui'
import {
  SearchCircleOutline,
  SparklesOutline,
  LayersOutline,
  ServerOutline,
  DocumentTextOutline,
  PowerOutline,
  PlayCircleOutline
} from '@vicons/ionicons5'

import useModelState from '@/composables/useModelState'
import { notify_service } from '@/services/notification_service'
import { match } from '@globalart/oxide'

// Тип для ключей моделей
type ModelKey = 'retriver' | 'generator'

// Композабл для состояния моделей
const { get_state, load_embedding_models, unload_embedding_models, load_generator_model, unload_generator_model} = useModelState()
const state = get_state()
const message = notify_service;

// Состояния загрузки для каждой модели
const loadingStates = reactive({
  retriver: false,
  generator: false
})

// Вычисляемые свойства
const loadedCount = computed(() => {
  if (state.value) {
    return (state.value.generator != null ? 1 : 0) + (state.value.retriver != null ? 1 : 0)
  } else {
    return 0
  }
})

const isAllLoaded = computed(() => loadedCount.value === 2)
const isPartiallyLoaded = computed(() => loadedCount.value > 0 && loadedCount.value < 2)
const areAllModelsLoaded = computed(() => isAllLoaded.value)
const isLoadingAny = computed(() =>
  loadingStates.retriver || loadingStates.generator
)

const iconColor = computed(() => {
  if (isAllLoaded.value) return '#18a058'
  if (isPartiallyLoaded.value) return '#2080f0'
  return '#909399'
})

// Методы
const getOverallStatusType = (): 'success' | 'warning' | 'default' => {
  if (isAllLoaded.value) return 'success'
  if (isPartiallyLoaded.value) return 'warning'
  return 'default'
}
const toggleModel = async (modelKey: ModelKey) => {
  if (!state.value) return
  
  const currentState = state.value[modelKey] != null
  loadingStates[modelKey] = true
  
  try 
  {
    if (currentState) 
    {
       
      // Выгружаем модель
       await match(modelKey, [
            ['generator', async () => await unload_generator_model()],
            ['retriver', async () => await unload_embedding_models()]
        ])
      message.success(`Модель ${getModelName(modelKey)} выгружена`)
    } else {
      // Загружаем модель
       await match(modelKey, [
            ['generator', async () => await load_generator_model()],
            ['retriver', async () => await load_embedding_models()]
        ])
      message.success(`Модель ${getModelName(modelKey)} загружена`)
    }
  } 
  catch (error) 
  {
    console.error(`Ошибка при переключении модели ${modelKey}:`, error)
    message.error(`Не удалось ${currentState ? 'выгрузить' : 'загрузить'} модель ${getModelName(modelKey)}`)
  } 
  finally 
  {
    loadingStates[modelKey] = false
  }
}

const toggleAllModels = async () => {
  const shouldLoadAll = !areAllModelsLoaded.value
  
  try {
    if (shouldLoadAll) {
      // Загружаем все модели
      loadingStates.retriver = true
      loadingStates.generator = true
      
      await Promise.all([
        load_generator_model(),
        load_embedding_models(),
      ])
      
      message.success('Все модели загружены')
    } else {
      // Выгружаем все модели
      loadingStates.retriver = true
      loadingStates.generator = true
      
      await Promise.all([
        unload_generator_model(),
        unload_embedding_models()
      ])
      
      message.success('Все модели выгружены')
    }
  } catch (error) {
    console.error('Ошибка при переключении всех моделей:', error)
    message.error(`Не удалось ${shouldLoadAll ? 'загрузить' : 'выгрузить'} все модели`)
  } finally {
    loadingStates.retriver = false
    loadingStates.generator = false
  }
}

const getModelName = (modelKey: ModelKey): string => {
  const names = {
    retriver: 'Ретривер',
    generator: 'Генератор'
  }
  return names[modelKey]
}

const getStatusClass = (modelKey: ModelKey): string => {
  if (!state.value) return 'status-unloaded'
  
  return state.value[modelKey] != null ? 'status-loaded' : 'status-unloaded'
}

// Форматирование байтов в читаемый вид
const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`
}
</script>

<style lang="scss" scoped>
.models-container {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  flex-wrap: nowrap;
}

.model-tag {
  cursor: pointer;
  transition: all 0.3s ease;
  font-weight: 500;

  &:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  &.status-tag {
    min-width: 55px;
    justify-content: center;
  }
}

// Popover content styles
.popover-content {
  padding: 16px;
}

.popover-header {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  margin-bottom: 16px;
}

.popover-title {
  flex: 1;

  .model-name {
    font-size: 16px;
    font-weight: 600;
    color: #1d1d1f;
    margin-bottom: 4px;
  }

  .model-description {
    font-size: 13px;
    color: #909399;
    line-height: 1.4;
  }
}

.popover-divider {
  height: 1px;
  background: linear-gradient(90deg, transparent, #e5e7eb 20%, #e5e7eb 80%, transparent);
  margin: 16px -16px;
}

.popover-control {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 16px;

  .control-label {
    font-size: 14px;
    color: #606266;
    font-weight: 500;
  }
}

.popover-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  background: linear-gradient(135deg, #f7f8fa 0%, #f0f2f5 100%);
  border-radius: 8px;
  border: 1px solid #e5e7eb;

  .info-item {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: #606266;
    font-weight: 500;
  }
}

.popover-section {
  margin-top: 16px;

  .section-title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    font-weight: 600;
    color: #606266;
    margin-bottom: 8px;
  }

  .system-prompt {
    font-size: 12px;
    line-height: 1.6;
    color: #606266;
    padding: 10px 12px;
    background: linear-gradient(135deg, #fafbfc 0%, #f5f7fa 100%);
    border-radius: 6px;
    border-left: 3px solid #2080f0;
    word-break: break-word;
    max-height: 150px;
    overflow-y: auto;

    &::-webkit-scrollbar {
      width: 6px;
    }

    &::-webkit-scrollbar-track {
      background: transparent;
    }

    &::-webkit-scrollbar-thumb {
      background: #dcdfe6;
      border-radius: 3px;

      &:hover {
        background: #c0c4cc;
      }
    }
  }
}

.popover-actions {
  margin-top: 16px;
}

// Анимации для тегов
:deep(.n-tag) {
  animation: fadeIn 0.3s ease-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

// Улучшенные стили для switch
:deep(.n-switch) {
  &.n-switch--active {
    .n-switch__rail {
      background: linear-gradient(135deg, #18a058 0%, #2ecc71 100%);
    }
  }
}
</style>