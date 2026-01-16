<template lang="pug">
n-card.employee-information-manager(:title="title" v-if="props.employee != null")
  template(#header-extra)
    n-button.edit-button(type="primary" @click="openAddModal")
      template(#icon)
        n-icon: add-icon
      | Добавить

  n-space(vertical :size="20")
    //- Фильтры
    .filter-panel
      n-input(
        v-model:value="searchQuery"
        placeholder="Поиск по свойству или значению"
        clearable
      )
        template(#prefix)
          n-icon: search-icon

    //- Список информации
    n-list(bordered)
      n-list-item(v-for="info in filteredInformation" :key="info.id")
        template(#suffix)
          n-space
            n-button(size="small" @click="openEditModal(info)") Редактировать
            n-button(size="small" type="error" @click="confirmDelete(info)") Удалить
        
        n-thing(:title="get_property(info.property).value" :description="info.value")
          

    //- Пустое состояние
    n-empty(
      v-if="filteredInformation.length === 0"
      :description="searchQuery ? 'Информация не найдена' : 'Информация отсуствует'"
    )
      template(#extra)
        n-button(size="small" @click="openAddModal") Добавить

//- Модальное окно добавления/редактирования
n-modal(
  v-model:show="showModal"
  :title="editingState ? 'Редактировать информацию' : 'Добавить информацию'"
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
    n-grid(:cols="2" :x-gap="24")
      n-gi(span="2")
        n-form-item(label="Свойство" path="property")
          n-select(
            filterable
            v-model:value="selected_property"
            @update:value="select_property_handle"
            :options="propertyOptions"
          )
      
      n-gi(span="2")
        n-form-item(label="Значение" path="value")
          n-input(
            v-model:value="formModel.value"
            type="textarea"
            placeholder="Значение свойства"
            :autosize="{ minRows: 3, maxRows: 6 }"
            maxlength="500"
          )

//- Диалог подтверждения удаления
n-modal(
  v-model:show="showDeleteConfirm"
  preset="dialog"
  type="error"
  title="Подтверждение удаления"
  :content="`Вы уверены, что хотите удалить свойство '${stateToDelete ? stateToDelete.property : ''}'?`"
  positive-text="Удалить"
  negative-text="Отмена"
  @positive-click="handleDelete"
  @negative-click="showDeleteConfirm = false"
)
</template>

<script lang="ts">
import { ref, computed, watch, defineProps, defineEmits } from 'vue'
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
  NGrid,
  NGi,
  NTag,
  NDatePicker,
  NSelect,
  NCheckbox,
  NInputNumber,
  type FormRules,
  type FormInst,
  type SelectOption,
  type FormItemRule
} from 'naive-ui'
import { AddOutline as AddIcon, SearchOutline as SearchIcon, ArrowForwardOutline} from '@vicons/ionicons5'
import { disease_ico, disease_red_ico, palm_ico } from '@/services/svg'
import { notify_service } from '@/services/notification_service'
import { http_sevice } from '@/services/http_service/http_service'
import { useDictionaries } from '@/composables/useDictionaries'
import SvgIcon from '@/components/SvgIcon.vue'
import { DateFormat, DateTime } from '@/services/date'
import type { EmployeeStatus } from '@/types/employee_status'
import { type EmployeeState, type Employee } from '@/types/employees'
import { insert_into_sorted_array } from '@/services/helpers'
import { type EmployeeInformation } from '@/types/employee_information'
</script>

<script lang="ts" setup>
const props = defineProps<{
  employee: Employee | null
}>()

const emit = defineEmits<{
  (e: 'saved', info: EmployeeInformation): void
  (e: 'deleted', info_id: string): void
}>()

const formRef = ref<FormInst | null>(null)
const { propertyOptions, get_property } = useDictionaries()

// Состояние
const information = ref<EmployeeInformation[] | null>()
const searchQuery = ref('')
const showModal = ref(false)
const showDeleteConfirm = ref(false)
const editingState = ref<EmployeeInformation | null>(null)
const stateToDelete = ref<EmployeeInformation | null>(null)
const selected_property = ref<string|null>(null)
const select_property_handle = (value: string, option: SelectOption) =>
{
  formModel.value.property = get_property(value)?.value ?? "";
}
// Модель формы
const formModel = ref({
  property: '',
  value: ''
})

// Правила валидации
const formRules: FormRules = {
  property: [
    {
      required: true,
      message: 'Название свойства обязательно',
      trigger: ['blur', 'change']
    },
    {
      min: 1,
      max: 100,
      message: 'Название должно быть от 1 до 100 символов',
      trigger: ['blur', 'change']
    }
  ],
  value: [
    {
      required: true,
      message: 'Значение свойства обязательно',
      trigger: ['blur', 'change']
    },
    {
      min: 1,
      max: 500,
      message: 'Значение должно быть от 1 до 500 символов',
      trigger: ['blur', 'change']
    }
  ]
}

// Computed свойства
const title = computed(() => `Информация: ${props.employee?.surname} ${props.employee?.first_name} ${props.employee?.second_name}`)
//фильрация
const filteredInformation = computed(() => 
{
  if (!information.value) return []
  
  let filtered = information.value
  
  if (searchQuery.value) 
  {
    const query = searchQuery.value.toLowerCase()
    filtered = filtered.filter(info => 
      info.property.toLowerCase().includes(query) ||
      info.value.toLowerCase().includes(query)
    )
  }
  
  return filtered.sort((a, b) => a.property.localeCompare(b.property))
})


// Методы
const openAddModal = async () => 
{
  editingState.value = null
  formModel.value = 
  {
    property: '',
    value: ''
  }
  showModal.value = true
}

const openEditModal = async (info: EmployeeInformation) => 
{
  editingState.value = info
  const current_property = get_property(info.property);
  selected_property.value = current_property?.id ?? null;
  formModel.value = 
  {
    property: current_property?.value ?? '',
    value: info.value
  }
  showModal.value = true
}

const confirmDelete = (state: EmployeeInformation) => 
{
  stateToDelete.value = state
  showDeleteConfirm.value = true
}

const handleDelete = async () => 
{
  if (!stateToDelete.value) return false
  const del = await http_sevice.employee_information.delete(stateToDelete.value.id);
  if(del)
  {
    information.value = information.value?.filter(s => s.id !== stateToDelete.value?.id)
    emit('deleted', stateToDelete.value.id);
    stateToDelete.value = null;
    notify_service.notify_success('Информация успешно удалена', '')
    showDeleteConfirm.value = false
    return true;
  }
  else
  {
    notify_service.notify_error('Ошибка при удалении информации', '');
    stateToDelete.value = null;
    return false
  }
}

const handleSave = async () => 
{
  const errors = await formRef.value?.validate();
  if (errors?.warnings) 
  {
    notify_service.notify_error('Пожалуйста, исправьте ошибки в форме', '')
    console.error("Ошибки при валидации данных ", errors.warnings);
    return false
  }
  else
  {
    if(editingState.value)
    {
      const saved = 
      {
        ...formModel.value,
        property: editingState.value.property,
        id: editingState.value.id,
        employee_id: editingState.value.employee_id,
      }
      const result = await http_sevice.employee_information.edit(saved.id, saved.property, saved.value);
      if(result && information.value)
      {
        const index = information.value.findIndex(s => s.id === editingState.value?.id)
        if (index !== -1) 
        {
          information.value[index] = saved;
          emit('saved', saved);
        }
        notify_service.notify_success('Информация обновлена', '');
        showModal.value = false
        resetForm()
        return true
      }
      else
      {
        notify_service.notify_error('Ошибка обновления информации', '');
        return false;
      }
    }
    else
    {
      const added = 
      {
        ...formModel.value,
        property: selected_property.value ?? '',
        employee_id: props.employee?.id as string,
      }
      const result = await http_sevice.employee_information.add(added.employee_id, added.property, added.value);
      if(result && information.value)
      {
        information.value = insert_into_sorted_array(information.value, result, information_compare);
        //states.value.push(result);
        //information.value.sort((a, b) => employees_states_compare(a, b));
        emit('saved', result);
        notify_service.notify_success('Информация добавлена', '');
        showModal.value = false
        resetForm()
        return true
      }
      else
      {
        return false;
      }
    }
  }
}

const information_compare = (a: EmployeeInformation, b: EmployeeInformation): number => 
{
  return b.property < a.property ? 1 : 0;
}

const handleCancel = () => 
{
  showModal.value = false
  resetForm()
  return true
}

const resetForm = () => 
{
  formModel.value = 
  {
    property: '',
    value: '',
  }
  editingState.value = null
}

// Загрузка данных при изменении employeeId
watch(() => props.employee, async (new_employee) => 
{
    if(new_employee)
    {
        const info = await http_sevice.employee_information.get(new_employee.id);
        if(info)
            information.value = info
    }  
}, { immediate: true })
</script>

<style lang="scss" scoped>
.employee-state-manager {
  max-width: 800px;
  margin: 0 auto;
}

.n-list 
{
  max-height: calc(100vh - 274px);
  overflow-y: auto;
}

.n-list-item {
  padding: 12px;
}

.n-avatar {
  font-weight: bold;
}

.status-stats {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 16px;
}
.filter-panel
{
  display: flex;
  flex-direction: row;
  gap: 10px;
}
.footer-panel
{
    align-items: center;
}
.edit-button
{
  margin-left: 10px;
}
</style>