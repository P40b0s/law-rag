<template lang="pug">
n-card.employee-status-manager(title="Управление статусами сотрудников")
  template(#header-extra)
    n-button(type="primary" @click="openAddModal")
      template(#icon)
        n-icon: add-icon
      | Добавить статус

  n-space(vertical :size="20")
    //- Поиск и фильтрация
    n-input(
      v-model:value="searchQuery"
      placeholder="Поиск по статусу или описанию..."
      clearable
    )
      template(#prefix)
        n-icon: search-icon
    loader(v-if="loading")
    //- Список статусов
    n-list(bordered v-else)
      n-list-item(v-for="status in filteredStatuses" :key="status.id")
        template(#suffix)
          n-space
            n-button(size="small" @click="openEditModal(status)") Редактировать
            n-button(size="small" type="error" @click="confirmDelete(status)") Удалить
        
        n-thing(:title="status.status" :description="status.description")
          template(#header-extra)
            n-tooltip
              template(#trigger)
                n-icon(:color="status.color"): Ellipse
              span Цвет статуса
          template(#avatar v-if="status.logo")
            svg-icon-native(:size="40" :svg="status.logo")
          
          template(#footer)
            n-space(size="small")
              n-tag(v-if="status.is_disease" size="small" type="error") Болезнь
              n-tag(v-if="status.tracing" size="small" type="warning") Отслеживание
              n-tag(v-if="status.on_work_place" size="small" type="success") Выполняет служебные обязанности

    //- Пустое состояние
    n-empty(
      v-if="filteredStatuses.length === 0"
      description="Статусы не найдены"
    )
      template(#extra)
        n-button(size="small" @click="openAddModal") Добавить статус

//- Модальное окно добавления/редактирования
n-modal(
  v-model:show="showModal"
  :title="editingStatus ? 'Редактировать статус' : 'Добавить статус'"
  preset="dialog"
  :style="{ width: '600px' }"
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
    n-grid(:cols="2" :x-gap="15")
      n-gi
        n-form-item(label="Статус" path="status")
          n-input(
            v-model:value="formModel.status"
            placeholder="Введите название статуса"
          )
        n-form-item(label="Описание" path="description")
          n-input(
            v-model:value="formModel.description"
            placeholder="Введите описание статуса"
            type="textarea"
            :autosize="{ minRows: 2, maxRows: 4 }"
          )
        .switch-place
          n-switch(v-model:value="formModel.is_disease" @update:value="diseaseHandleChange")
          label-with-description(name="Заболевание?" description="Необходимо отметить если статус является заболеванием")
        .switch-place
          n-switch(v-model:value="formModel.tracing" @update:value="traceHandleChange")
          label-with-description(name="Отслеживается?" description="Если заболевание требует особого режима отслеживания")
        .switch-place
          n-switch(v-model:value="formModel.on_work_place" @update:value="onWorkPlaceHandleChange")
          label-with-description(name="Выполняет свои обязанности?" description="Если во время действия статуса сотрудник выполняет свои обязанности")
        color-picker(:value="formModel.color" @update:value="(v: string) => formModel.color = v")
      n-gi.logo
        svg-selector(v-model:svg="formModel.logo")

//- Диалог подтверждения удаления
n-modal(
  v-model:show="showDeleteConfirm"
  preset="dialog"
  type="error"
  title="Подтверждение удаления"
  content="Вы уверены, что хотите удалить этот статус?"
  positive-text="Удалить"
  negative-text="Отмена"
  @positive-click="handleDelete"
  @negative-click="showDeleteConfirm = false"
)
</template>

<script lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
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
  NColorPicker,
  NForm,
  NFormItem,
  NGrid,
  NGi,
  NSwitch,
  NTag,
  NTooltip,
  type FormRules,
  type FormInst
} from 'naive-ui'
import { LabelWithDescription } from './label_with_description';
import { disease_ico, disease_red_ico, palm_ico } from '@/services/svg';
import { AddOutline as AddIcon, Ellipse, SearchOutline as SearchIcon } from '@vicons/ionicons5'
import { type EmployeeStatus } from '@/types/employee_status'
import { notify_service } from '@/services/notification_service'
import { http_sevice } from '@/services/http_service/http_service'
import Loader from './Loader.vue'
import { useDictionaries } from '@/composables/useDictionaries'
import SvgIcon from './SvgIcon.vue';
import SvgSelector from './SvgSelector.vue';
import SvgIconNative from './SvgIconNative.vue';
import ColorPicker from './ColorPicker.vue';
</script>

<script lang="ts" setup>
const formRef = ref<FormInst | null>(null)

const {statuses, add_status, edit_status, delete_status, loading} = useDictionaries();
// Состояние
const statuses_ref = computed(() => Array.from(statuses.value.values()))
const searchQuery = ref('')
const showModal = ref(false)
const showDeleteConfirm = ref(false)
const editingStatus = ref<EmployeeStatus | null>(null)
const statusToDelete = ref<EmployeeStatus | null>(null)

// Модель формы
const formModel = ref({
  status: '',
  description: '',
  logo: null as string|null,
  color: '#89DAD3ab' as string,
  is_disease: false,
  tracing: false,
  on_work_place: false
})
const tag_color = computed(() =>
{
  return {
    
    borderColor: formModel.value.color,
    textColor: formModel.value.color,
  }
})
//правила чтобы не поставили взаимоисключающие флаги
const diseaseHandleChange = (value: boolean) =>
{
  if(value)
  {
    formModel.value.on_work_place = false;
  }
  else
  {
    formModel.value.tracing = false;
  }
}
const traceHandleChange = (value: boolean) =>
{
  if(value)
  {
    formModel.value.is_disease = true;
    formModel.value.on_work_place = false;
  }
  else
  {
    formModel.value.is_disease = false;
  }
}
const onWorkPlaceHandleChange = (value: boolean) =>
{
  if(value)
  {
    formModel.value.is_disease = false;
    formModel.value.tracing = false;
  }
}

// Правила валидации
const formRules: FormRules = {
  status: [
    {
      required: true,
      message: 'Название статуса обязательно',
      trigger: ['blur', 'input']
    },
    {
      min: 2,
      message: 'Название должно содержать минимум 2 символа',
      trigger: ['blur', 'input']
    }
  ],
  description: [
    {
      required: true,
      message: 'Описание статуса обязательно',
      trigger: ['blur', 'input']
    }
  ]
}

// Отфильтрованные статусы
const filteredStatuses = computed(() => {
  if (!searchQuery.value) {
    return statuses_ref.value
  }

  const query = searchQuery.value.toLowerCase()
  return statuses_ref.value.filter(status =>
    status.status.toLowerCase().includes(query) ||
    status.description.toLowerCase().includes(query)
  )
})

// Открытие модального окна для добавления
const openAddModal = () => 
{
  editingStatus.value = null
  formModel.value = {
    status: '',
    description: '',
    logo: null,
    color: '#41C6BD',
    is_disease: false,
    tracing: false,
    on_work_place: false
  }
  showModal.value = true
}

// Открытие модального окна для редактирования
const openEditModal = (status: EmployeeStatus) => 
{
  editingStatus.value = status
  formModel.value = 
  {
    status: status.status,
    description: status.description,
    logo: status.logo ?? null,
    color: status.color,
    is_disease: status.is_disease,
    tracing: status.tracing,
    on_work_place: status.on_work_place
  }
  showModal.value = true
}

// Подготовка к удалению
const confirmDelete = (status: EmployeeStatus) => 
{
  statusToDelete.value = status
  showDeleteConfirm.value = true
}

// Удаление статуса
const handleDelete = async () => 
{
  if (statusToDelete.value) 
  {
    //await http_sevice.employee_status_service.delete(statusToDelete.value.id)
    //statuses_ref.value = statuses_ref.value.filter(s => s.id !== statusToDelete.value!.id)
    //notify_service.notify_success('Статус удален', '')
    await delete_status(statusToDelete.value.id);
    statusToDelete.value = null
  }
  showDeleteConfirm.value = false
}

// Сохранение статуса
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
      if (editingStatus.value) 
      {
        // Редактирование существующего статуса
        await edit_status(editingStatus.value.id,
          formModel.value.status,
          formModel.value.description,
          formModel.value.is_disease,
          formModel.value.tracing,
          formModel.value.on_work_place,
          formModel.value.color,
          formModel.value.logo ?? undefined);
      }
      else 
      {
        await add_status(formModel.value.status,
          formModel.value.description,
          formModel.value.is_disease,
          formModel.value.tracing,
          formModel.value.on_work_place,
          formModel.value.color,
          formModel.value.logo ?? undefined);
      }

      showModal.value = false
      resetForm()
      return true
    } 
    catch (error) 
    {
      notify_service.notify_error('Ошибка при сохранении статуса', '')
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
  formModel.value = 
  {
    status: '',
    description: '',
    is_disease: false,
    color: '#41C6BD',
    logo: null,
    tracing: false,
    on_work_place: false
  }
  editingStatus.value = null
}
</script>

<style lang="scss" scoped>
.employee-status-manager {
  max-width: 800px;
  margin: 0 auto;
}

.n-list-item {
  padding: 12px;
}

.n-avatar {
  font-weight: bold;
}
.switch-place
{
  display: flex;
  flex-direction: row;
  margin-top: 10px;
  gap: 10px;

}
.logo
{
  display: flex;
  justify-content: center;
  max-height: 250px;
}
.n-list 
{
  max-height: calc(100vh - 220px);
  overflow-y: auto;
}

</style>