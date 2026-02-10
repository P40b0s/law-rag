<template lang="pug">
n-card(title="Добавить документ" :bordered="false")
  n-form(
    ref="formRef"
    :model="formData"
    :rules="formRules"
    label-placement="left"
    label-width="140"
  )
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

    n-form-item
      n-space(:size="12")
        n-button(
          type="primary"
          :loading="isLoading"
          :disabled="!formData.sign_date || !formData.number"
          @click="handleAddDocument"
        )
          template(#icon)
            n-icon(:component="AddOutline")
          | Добавить документ

        n-button(
          v-if="formData.sign_date || formData.number"
          @click="handleClear"
        )
          | Очистить
</template>

<script setup lang="ts">
import { ref } from 'vue'
import {
  NCard,
  NForm,
  NFormItem,
  NInput,
  NDatePicker,
  NButton,
  NSpace,
  NIcon,
  type FormInst,
  type FormRules
} from 'naive-ui'
import {
  DocumentTextOutline,
  AddOutline
} from '@vicons/ionicons5'
import { http_sevice } from '@/services/http_service/http_service'
import { notify_service } from '@/services/notification_service'
import { DateTime } from '@/services/date'
import type { Document } from '@/types/document'

// Emits
const emit = defineEmits<{
  documentAdded: [document: Document]
}>()

// Reactive state
const formRef = ref<FormInst | null>(null)
const isLoading = ref(false)
const formData = ref({
  sign_date: null as number | null,
  number: ''
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
  }
}

// Methods
const handleAddDocument = async () => {
  if (!formRef.value) return

  try {
    await formRef.value.validate()

    if (!formData.value.sign_date || !formData.value.number) {
      notify_service.warning('Заполните все поля', 'Необходимо указать дату подписания и номер документа')
      return
    }

    isLoading.value = true

    // Конвертируем timestamp в формат DateTime
    const date = new Date(formData.value.sign_date)
    const dateTime = DateTime.parse(date)

    // Запрашиваем документ
    const document = await http_sevice.documents_service.request_document(dateTime, formData.value.number)

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

const handleClear = () => 
{
  formData.value = {
    sign_date: null,
    number: ''
  }
}
</script>

<style scoped>
:deep(.n-card) {
  margin-bottom: 24px;
}
</style>
