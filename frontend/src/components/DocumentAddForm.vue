<template lang="pug">
n-card(:bordered="false")
  n-form(
    ref="formRef"
    :model="formData"
    :rules="formRules"
    label-placement="left"
    label-width="140"
  )
    n-form-item(label="Парсер" path="parser")
       n-input(
        v-model:value="formData.parser"
        placeholder="Например: 23.05.2025 № 108-ФЗ"
        clearable
        @update:value="parserHandler"
      )
        template(#prefix)
          n-icon(:component="BarcodeOutline")


    n-form-item(label="Дата подписания" path="sign_date")
      n-date-picker(
        v-model:value="formData.sign_date"
        type="date"
        placeholder="Выберите дату подписания"
        :style="{ width: '100%' }"
        format="dd.MM.yyyy"
        value-format="yyyy-MM-dd"
      )

    n-form-item(label="Номер документа" path="number")
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
        :render-label="renderCollectionLabel"
        :render-tag="renderCollectionTag"
        clearable
      )

    n-form-item
      n-space(:size="12")
        n-button(
          type="primary"
          :loading="isLoading"
          :disabled="!formData.sign_date || !formData.number || !formData.collection_id"
          @click="handleAddDocument"
        )
          template(#icon)
            n-icon(:component="AddOutline")
          | Добавить документ

        n-button(
          v-if="formData.sign_date || formData.number || formData.collection_id"
          @click="handleClear"
        )
          | Очистить
</template>

<script setup lang="ts">
import { ref, h } from 'vue'
import {
  NCard,
  NForm,
  NFormItem,
  NInput,
  NDatePicker,
  NButton,
  NSpace,
  NIcon,
  NSelect,
  NText,
  type FormInst,
  type FormRules,
  type SelectRenderLabel
} from 'naive-ui'
import {
  DocumentTextOutline,
  AddOutline,
  BarcodeOutline
} from '@vicons/ionicons5'
import { notify_service } from '@/services/notification_service'
import { DateTime } from '@/services/date'
import type { Document } from '@/types/document'
import useCollections from '@/composables/useCollections'
import useEmbeddings from '@/composables/useEmbeddings'

// Emits
const emit = defineEmits<{
  documentAdded: [document: Document]
}>()

// Collections
const { collectionOptions, loading: collectionsLoading } = useCollections()
const {request_document} = useEmbeddings();
// Reactive state
const formRef = ref<FormInst | null>(null)
const isLoading = ref(false)
const formData = ref({
  sign_date: null as number | null,
  number: '',
  collection_id: null as string | null,
  parser: ''
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


// Render label for collection select (dropdown options)
const renderCollectionLabel: SelectRenderLabel = (option) => {
  return h('div', { style: { padding: '4px 0' } }, [
    h('div', { style: { fontWeight: 600, marginBottom: '4px' } }, option.label as string),
    h(NText, {
      depth: 3,
      style: {
        fontSize: '12px',
        lineHeight: '1.4',
        whiteSpace: 'normal',
        wordBreak: 'break-word',
        display: 'block'
      }
    }, { default: () => option.description })
  ])
}

// Render tag for selected value (compact view)
const renderCollectionTag = ({ option }: { option: any }) => {
  return h('span', {
    style: {
      overflow: 'hidden',
      textOverflow: 'ellipsis',
      whiteSpace: 'nowrap',
      display: 'block'
    }
  }, option.label as string)
}

// Methods
const handleAddDocument = async () => {
  if (!formRef.value) return

  try {
    await formRef.value.validate()

    if (!formData.value.sign_date || !formData.value.number || !formData.value.collection_id) {
      notify_service.warning('Заполните все поля', 'Необходимо указать дату подписания, номер документа и коллекцию')
      return
    }

    isLoading.value = true

    // Конвертируем timestamp в формат DateTime
    const date = new Date(formData.value.sign_date)
    const dateTime = DateTime.parse(date)

    // Запрашиваем документ
    const document = await request_document(dateTime, formData.value.number, formData.value.collection_id)

    if (document) 
    {
      notify_service.success('Документ добавлен', `Документ ${formData.value.number} успешно загружен`)

      // Отправляем событие родителю
      emit('documentAdded', document)

      // Очищаем форму
      handleClear()
    }
  } 
  catch (error) 
  {
    console.error('Error adding document:', error)
    notify_service.error('Ошибка', 'Не удалось загрузить документ')
  } 
  finally 
  {
    isLoading.value = false
  }
}
//TODO не работает, не забыть исправить!
const parserHandler = (value: string | [string, string]): void =>
{
  if (typeof value === 'string') 
  {
    const regex = /^(\d{2}\.\d{2}\.\d{4})\s+№\s*(.+)$/;
    const match = value.match(regex);
    
    formData.value.sign_date = match ? DateTime.parse(match[1]).date.getTime() : null;
    formData.value.number = match ? match[2] : "";
  }
}

const handleClear = () =>
{
  formData.value = {
    sign_date: null,
    number: '',
    collection_id: null,
    parser: ''
  }
}
</script>

<style scoped>
:deep(.n-card) {
  margin-bottom: 24px;
}

</style>
