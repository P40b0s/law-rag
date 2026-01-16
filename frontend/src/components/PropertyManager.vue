<template lang="pug">
n-card.property-manager(title="Управление свойствами")
  template(#header-extra)
    n-button(type="primary" @click="openAddModal")
      template(#icon)
        n-icon: add-icon
      | Добавить

  n-space(vertical :size="20")
    //- Поиск и фильтрация
    n-input(
      v-model:value="searchQuery"
      placeholder="Поиск по названию..."
      clearable
    )
      template(#prefix)
        n-icon: search-icon

    //- Список отделов
    n-list(bordered)
      n-list-item(v-for="value in filtered" :key="value.id")
        template(#suffix)
          n-space
            n-tooltip Удалить
              template(#trigger)
                n-button(round text @click="confirmDelete(value)")
                  template(#icon)
                    n-icon(:size="25" color="#ec3c36"): TrashBin
        n-thing(:title="value.value")
        template(#prefix)
          n-tooltip Редактировать
            template(#trigger)
              n-button(round text @click="openEditModal(value)")
                template(#icon)
                  n-icon(:size="25" color="#82e873"): EditIcon
          
          
            

    //- Пустое состояние
    n-empty(
      v-if="filtered.length === 0"
      description="Свойства не найдены"
    )
      template(#extra)
        n-button(size="small" @click="openAddModal") Добавить свойство

//- Модальное окно добавления/редактирования
n-modal(
  v-model:show="showModal"
  :title="editingValue ? 'Редактировать свойство' : 'Добавить свойство'"
  preset="dialog"
  positive-text="Сохранить"
  negative-text="Отмена"
  @positive-click="handleSave"
  @negative-click="handleCancel"
)
  n-form(
    ref="formRef"
    :model="formModel"
    :rules="formRules"
    label-placement="top"
  )
    n-form-item(label="Название свойства" path="value")
      n-input(
        v-model:value="formModel.value"
        placeholder="Введите название свойства"
      )

//- Диалог подтверждения удаления
n-modal(
  v-model:show="showDeleteConfirm"
  preset="dialog"
  type="error"
  title="Подтверждение удаления"
  content="Вы уверены, что хотите удалить это свойство?"
  positive-text="Удалить"
  negative-text="Отмена"
  @positive-click="handleDelete"
  @negative-click="showDeleteConfirm = false"
)
</template>

<script lang="ts">
import { ref, computed, onMounted } from 'vue'
import {
  NCard,
  NButton,
  NIcon,
  NSpace,
  NInput,
  NList,
  NListItem,
  NThing,
  NAvatar,
  NEmpty,
  NModal,
  NForm,
  NFormItem,
  NTooltip,
  useMessage,
  type FormRules,
  type FormInst
} from 'naive-ui'
import { AddOutline as AddIcon, SearchOutline as SearchIcon, TrashBin } from '@vicons/ionicons5'
import { Edit as EditIcon } from '@vicons/carbon'

import { type Dictionary } from '@/types/dictionary'
import { notify_service } from '@/services/notification_service'
import { http_sevice } from '@/services/http_service/http_service'
import { useDictionaries } from '../composables/useDictionaries';
</script>

<script lang="ts" setup>
const formRef = ref<FormInst | null>(null)
const {properties, add_property, delete_property, edit_property} = useDictionaries()
// Состояние
const values = computed(() => Array.from(properties.value.values()));
const searchQuery = ref('')
const showModal = ref(false)
const showDeleteConfirm = ref(false)
const editingValue = ref<Dictionary | null>(null)
const valueToDelete = ref<Dictionary | null>(null)

// Модель формы
const formModel = ref({
  value: ''
})

// Правила валидации
const formRules: FormRules = {
  value: [
    {
      required: true,
      message: 'Название свойства обязательно',
      trigger: ['blur', 'input']
    },
    {
      min: 2,
      message: 'Название должно содержать минимум 2 символа',
      trigger: ['blur', 'input']
    }
  ]
}

// Отфильтрованные отделы
const filtered= computed(() => 
{
  if (!searchQuery.value) 
  {
    return values.value
  }

  const query = searchQuery.value.toLowerCase()
  return values.value.filter(v =>
    v.value.toLowerCase().includes(query)
  )
})

// Загрузка данных
onMounted(async () => 
{
  //departments_ref.value = await http_sevice.department_service.get()
})

// Открытие модального окна для добавления
const openAddModal = () => 
{
  editingValue.value = null
  formModel.value = 
  {
    value: ''
  }
  showModal.value = true
}

// Открытие модального окна для редактирования
const openEditModal = (v: Dictionary) => 
{
  editingValue.value = v
  formModel.value = 
  {
    value: v.value
  }
  showModal.value = true
}

// Подготовка к удалению
const confirmDelete = (v: Dictionary) => 
{
  valueToDelete.value = v
  showDeleteConfirm.value = true
}

// Удаление отдела
const handleDelete = async () => 
{
  if (valueToDelete.value) 
  {
    await delete_property(valueToDelete.value.id);
    valueToDelete.value = null
  }
  showDeleteConfirm.value = false
}

// Сохранение отдела
const handleSave = () => 
{
  formRef.value?.validate(async (errors) => 
  {
    if (errors) 
    {
      notify_service.notify_error('Пожалуйста, исправьте ошибки в форме', '')
      return false
    }

    try {
      if (editingValue.value) 
      {
        await edit_property(editingValue.value.id,formModel.value.value);
      } 
      else 
      {
        await add_property(formModel.value.value);
      }

      showModal.value = false
      resetForm()
      return true
    } 
    catch (error) 
    {
      notify_service.notify_error('Ошибка при сохранении свойства', '')
      return false
    }
  })
}

// Отмена редактирования
const handleCancel = () => 
{
  showModal.value = false
  resetForm()
}

// Сброс формы
const resetForm = () => 
{
  formModel.value = {
    value: ''
  }
  editingValue.value = null
}
</script>

<style lang="scss" scoped>
.property-manager {
  max-width: 800px;
  min-width: 600px;
  margin: 0 auto;
}

.n-list 
{
  max-height: calc(100vh - 220px);
  overflow-y: auto;
}
.n-list-item {
  padding: 12px;
}
</style>