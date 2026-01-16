<template lang="pug">
n-card.department-manager(title="Управление отделами")
  template(#header-extra)
    n-button(type="primary" @click="openAddModal")
      template(#icon)
        n-icon: add-icon
      | Добавить отдел

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
      n-list-item(v-for="department in filteredDepartments" :key="department.id")
        template(#suffix)
          n-space
            n-tooltip Удалить
              template(#trigger)
                n-button(round text @click="confirmDelete(department)")
                  template(#icon)
                    n-icon(:size="25" color="#ec3c36"): TrashBin
        n-thing(:title="department.value")
          template(#header-extra)
            n-tooltip
              template(#trigger)
                span {{department.weight}}
              div Сортировочный вес
        template(#prefix)
          n-tooltip Редактировать
            template(#trigger)
              n-button(round text @click="openEditModal(department)")
                template(#icon)
                  n-icon(:size="25" color="#82e873"): EditIcon

    //- Пустое состояние
    n-empty(
      v-if="filteredDepartments.length === 0"
      description="Отделы не найдены"
    )
      template(#extra)
        n-button(size="small" @click="openAddModal") Добавить отдел

//- Модальное окно добавления/редактирования
n-modal(
  v-model:show="showModal"
  :title="editingDepartment ? 'Редактировать отдел' : 'Добавить отдел'"
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
    n-form-item(label="Название отдела" path="value")
      n-input(
        v-model:value="formModel.value"
        placeholder="Введите название отдела"
      )

    n-form-item(label="Сортировка" path="weight")
      n-input-number(
        v-model:value="formModel.weight"
        :min="0"
        :max="255"
        placeholder="Введите вес"
      )

//- Диалог подтверждения удаления
n-modal(
  v-model:show="showDeleteConfirm"
  preset="dialog"
  type="error"
  title="Подтверждение удаления"
  content="Вы уверены, что хотите удалить этот отдел?"
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
  NInputNumber,
  type FormRules,
  type FormInst
} from 'naive-ui'
import { AddOutline as AddIcon, SearchOutline as SearchIcon, TrashBin } from '@vicons/ionicons5'
import { Edit as EditIcon } from '@vicons/carbon'
import { type DictionaryWithWeight as Dictionary } from '@/types/dictionary'
import { notify_service } from '@/services/notification_service'
import { http_sevice } from '@/services/http_service/http_service'
import { useDictionaries } from '../composables/useDictionaries';
</script>

<script lang="ts" setup>
const formRef = ref<FormInst | null>(null)
const {departments, add_department, delete_department, edit_department} = useDictionaries()
// Состояние
const departments_ref = computed(() => Array.from(departments.value.values()));
const searchQuery = ref('')
const showModal = ref(false)
const showDeleteConfirm = ref(false)
const editingDepartment = ref<Dictionary | null>(null)
const departmentToDelete = ref<Dictionary | null>(null)

// Модель формы
const formModel = ref({
  value: '',
  weight: 0
})

// Правила валидации
const formRules: FormRules = {
  value: [
    {
      required: true,
      message: 'Название отдела обязательно',
      trigger: ['blur', 'input']
    },
    {
      min: 2,
      message: 'Название должно содержать минимум 2 символа',
      trigger: ['blur', 'input']
    }
  ],
  weight: [
    {
      type: 'number',
      required: true,
      message: 'Необходимо указать вес, для сортировки',
      trigger: ['blur', 'input']
    }
  ]
}

// Отфильтрованные отделы
const filteredDepartments = computed(() => 
{
  if (!searchQuery.value) 
  {
    return departments_ref.value
  }

  const query = searchQuery.value.toLowerCase()
  return departments_ref.value.filter(department =>
    department.value.toLowerCase().includes(query)
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
  editingDepartment.value = null
  formModel.value = 
  {
    value: '',
    weight: 0
  }
  showModal.value = true
}

// Открытие модального окна для редактирования
const openEditModal = (department: Dictionary) => 
{
  editingDepartment.value = department
  formModel.value = 
  {
    value: department.value,
    weight: department.weight
  }
  showModal.value = true
}

// Подготовка к удалению
const confirmDelete = (department: Dictionary) => 
{
  departmentToDelete.value = department
  showDeleteConfirm.value = true
}

// Удаление отдела
const handleDelete = async () => 
{
  if (departmentToDelete.value) 
  {
    //await http_sevice.department_service.delete(departmentToDelete.value.id)
    //departments_ref.value = departments_ref.value.filter(d => d.id !== departmentToDelete.value!.id)
    //notify_service.notify_success('Отдел удален', '')
    await delete_department(departmentToDelete.value.id);
    departmentToDelete.value = null
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
      if (editingDepartment.value) 
      {
        await edit_department(editingDepartment.value.id,formModel.value.value, formModel.value.weight);
      } 
      else 
      {
        // Добавление нового отдела
        // const newDepartment: DepartmentDictionary = 
        // {
        //   id: '',
        //   value: formModel.value.value
        // }
        // const added = await http_sevice.department_service.add(newDepartment.value)
        // if (added) 
        // {
        //   departments_ref.value.push(added)
        //   notify_service.notify_success('Отдел добавлен', '')
        // }
        await add_department(formModel.value.value, formModel.value.weight);
      }

      showModal.value = false
      resetForm()
      return true
    } 
    catch (error) 
    {
      notify_service.notify_error('Ошибка при сохранении отдела', '')
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
    value: '',
    weight: 0
  }
  editingDepartment.value = null
}
</script>

<style lang="scss" scoped>
.department-manager {
  max-width: 800px;
  min-width: 600px;
  margin: 0 auto;
}
.n-list-item {
  padding: 12px;
}
.n-list 
{
  max-height: calc(100vh - 220px);
  overflow-y: auto;
}
</style>