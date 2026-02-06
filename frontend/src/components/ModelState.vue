<template lang="pug">
n-popover(trigger="hover" :show-arrow="false" placement="bottom" width="400" v-if="state")
  template(#trigger)
    .status-badge(:class="statusClass")
      n-icon(size="16" :color="iconColor")
        component(:is="statusIcon")
      span.status-text {{ statusText }}
  
  .popover-content
    h3.popover-title Управление моделями
    .popover-grid
      .popover-item
        .popover-label Ретривер:
        .popover-controls
          n-switch(
            :value="state.retriver"
            @update:value="toggleModel('retriver')"
            :loading="loadingStates.retriver"
            size="small"
            :disabled="loadingStates.generator || loadingStates.reranker"
          )
          .popover-status(:class="getStatusClass('retriver')")
            | {{ state.retriver ? 'Загружен' : 'Выгружен' }}
      .popover-item
        .popover-label Генератор:
        .popover-controls
          n-switch(
            :value="state.generator"
            @update:value="toggleModel('generator')"
            :loading="loadingStates.generator"
            size="small"
            :disabled="loadingStates.retriver || loadingStates.reranker"
          )
          .popover-status(:class="getStatusClass('generator')")
            | {{ state.generator ? 'Загружен' : 'Выгружен' }}
    
    .popover-divider
    
    .popover-section(v-if="state.system_prompt")
      h4.popover-subtitle Системный промпт:
      .popover-system-prompt {{ state.system_prompt }}
    
    .popover-section(v-if="state.model_size")
      h4.popover-subtitle Размер модели:
      .popover-size {{ formatBytes(state.model_size) }}
    
    .popover-footer
      .popover-actions
        n-button(
          size="small"
          type="primary"
          @click="toggleAllModels"
          :loading="isLoadingAny"
          :disabled="isLoadingAny"
        )
          | {{ areAllModelsLoaded ? 'Выгрузить все' : 'Загрузить все' }}
        n-tag(type="info" size="small" round)
          | {{ loadedCount }}/2 моделей загружено
</template>

<script lang="ts" setup>
import { computed, ref, reactive } from 'vue'
import { NPopover, NIcon, NTag, NSwitch, NButton} from 'naive-ui'
import {
  CheckmarkCircleOutline,
  AlertCircleOutline,
  TimeOutline
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
    return (state.value.generator ? 1 : 0) + (state.value.retriver ? 1 : 0)
  } else {
    return 0
  }
})

const isAllLoaded = computed(() => loadedCount.value === 2)
const isPartiallyLoaded = computed(() => loadedCount.value > 0 && loadedCount.value < 2)
const isNoneLoaded = computed(() => loadedCount.value === 0)
const areAllModelsLoaded = computed(() => isAllLoaded.value)
const isLoadingAny = computed(() => 
  loadingStates.retriver || loadingStates.generator
)

const statusClass = computed(() => ({
  'status-all-loaded': isAllLoaded.value,
  'status-partial': isPartiallyLoaded.value,
  'status-none': isNoneLoaded.value
}))

const statusIcon = computed(() => {
  if (isAllLoaded.value) return CheckmarkCircleOutline
  if (isPartiallyLoaded.value) return TimeOutline
  return AlertCircleOutline
})

const iconColor = computed(() => {
  if (isAllLoaded.value) return '#18a058'
  if (isPartiallyLoaded.value) return '#2080f0'
  return '#d03050'
})

const statusText = computed(() => {
  if (isAllLoaded.value) return 'Все модели загружены'
  if (isPartiallyLoaded.value) return `Частично (${loadedCount.value}/2)`
  return 'Модели не загружены'
})

// Методы
const toggleModel = async (modelKey: ModelKey) => {
  if (!state.value) return
  
  const currentState = state.value[modelKey]
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
  
  return state.value[modelKey] ? 'status-loaded' : 'status-unloaded'
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
.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 20px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  user-select: none;
  
  &:hover {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }
  
  &.status-all-loaded {
    background: rgba(24, 160, 88, 0.1);
    color: #18a058;
    border: 1px solid rgba(24, 160, 88, 0.2);
  }
  
  &.status-partial {
    background: rgba(32, 128, 240, 0.1);
    color: #2080f0;
    border: 1px solid rgba(32, 128, 240, 0.2);
  }
  
  &.status-none {
    background: rgba(208, 48, 80, 0.1);
    color: #d03050;
    border: 1px solid rgba(208, 48, 80, 0.2);
  }
}

.status-text {
  white-space: nowrap;
}

.popover-content {
  padding: 8px 0;
}

.popover-title {
  margin: 0 0 16px 0;
  padding: 0 16px;
  font-size: 16px;
  font-weight: 600;
  color: #1d1d1f;
}

.popover-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 16px;
  padding: 0 16px;
}

.popover-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.popover-label {
  font-size: 14px;
  color: #515767;
  font-weight: 500;
  flex-shrink: 0;
}

.popover-controls {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}

.popover-status {
  font-size: 12px;
  font-weight: 500;
  min-width: 70px;
  text-align: right;
  
  &.status-loaded {
    color: #18a058;
  }
  
  &.status-unloaded {
    color: #d03050;
  }
}

.popover-divider {
  height: 1px;
  background: #e5e7eb;
  margin: 16px 16px;
}

.popover-section {
  padding: 0 16px;
  margin-bottom: 12px;
  
  &:last-child {
    margin-bottom: 0;
  }
}

.popover-subtitle {
  margin: 0 0 8px 0;
  font-size: 14px;
  font-weight: 600;
  color: #1d1d1f;
}

.popover-system-prompt {
  font-size: 13px;
  line-height: 1.5;
  color: #515767;
  padding: 8px 12px;
  background: #f7f8fa;
  border-radius: 6px;
  border-left: 3px solid #2080f0;
  word-break: break-word;
  max-height: 120px;
  overflow-y: auto;
}

.popover-size {
  font-size: 14px;
  color: #515767;
  font-weight: 500;
}

.popover-footer {
  padding: 0 16px;
  margin-top: 12px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.popover-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}
</style>