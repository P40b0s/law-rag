<template lang="pug">
n-card.employee-manager(title="Сотрудники")
  template(#header-extra)
    n-button.edit-button(type="primary" @click="openAddModal")
      template(#icon)
        n-icon: add-icon
      | Добавить

  n-space(vertical :size="20")
    //- Поиск и фильтрация
    n-input(
      v-model:value="searchQuery"
      placeholder="Поиск по ФИО..."
      clearable
    )
      template(#prefix)
        n-icon: search-icon
    //- Фильтры
    .filter-panel
      n-select(v-model:value="selected_department"
              clearable
              :options="departmentOptions"
              placeholder="Фильтр по отделу")
      n-select(v-model:value="selected_status"
            clearable
            :options="current_status_options"
            placeholder="Фильтр по состоянию")
    n-space
      string-statistic(:employees="filteredEmployees")
    //- Список сотрудников
    n-list(bordered)
      n-list-item(v-for="employee in filteredEmployees" :key="employee.id")
        template(#suffix)
          n-space
            n-button(size="small" @click="select_statuses_handle(employee)") Информация
            n-button(size="small" @click="openEditModal(employee)") Редактировать
            n-button(size="small" type="error" @click="confirmDelete(employee)") Удалить
        
        n-thing(:title="fullName(employee)")
          template(#description)
            div {{employeeDescription(employee)}}

          template(#header-extra)
            .birthday 
              n-tag(size="medium" type="info")  {{formatDate(employee.birthday)}} г.р.
              n-tag(size="medium" type="info")  {{years_count(employee.birthday)}} л.

          template(#avatar)
            n-avatar(v-if="is_birthday(employee.birthday)" size="small" type="warning")
              svg-icon(:size="34" :svg="birthday_cake_ico")
            n-avatar(v-else size="small") 
              | {{ getInitials(employee) }}
          
          template(#footer)
            n-divider
            n-space(size="small")
              template(v-if="get_status(employee.status_id)")
                template(v-if="get_status(employee.status_id).is_disease")
                  tag-with-progress(
                    :color="get_status(employee.status_id).color"
                    :logo="get_status(employee.status_id).logo"
                    :status_name="get_status(employee.status_id)?.status" 
                    :date_from="employee.state_start_date" 
                    :date_to="employee.state_end_date")
                template(v-else)
                    n-tag(v-if="get_status(employee.status_id).on_work_place" size="medium" type="success") На рабочем месте
                    tag-with-progress(
                      :color="get_status(employee.status_id).color"
                      :logo="get_status(employee.status_id).logo"
                      :status_name="get_status(employee.status_id)?.status"
                      :date_from="employee.state_start_date" 
                      :date_to="employee.state_end_date")
              n-tag(v-else size="medium" type="success") На рабочем месте
              n-tag(v-if="is_birthday(employee.birthday)" size="medium" type="info") Сегодня день рождения!
                template(#icon)
                  n-icon(:component="FaceCool")

    //- Пустое состояние
    n-empty(
      v-if="filteredEmployees.length === 0"
      description="Сотрудники не найдены"
    )
      template(#extra)
        n-button(size="small" @click="openAddModal") Добавить сотрудника

//- Модальное окно добавления/редактирования
n-modal(
  v-model:show="showModal"
  :title="editingEmployee ? 'Редактировать сотрудника' : 'Добавить сотрудника'"
  preset="dialog"
  :style="{ width: '700px' }"
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
      n-gi
        n-form-item(label="Фамилия" path="surname")
          n-input(
            v-model:value="formModel.surname"
            placeholder="Введите фамилию"
          )
      n-gi
        n-form-item(label="Имя" path="first_name")
          n-input(
            v-model:value="formModel.first_name"
            placeholder="Введите имя"
          )
      n-gi
        n-form-item(label="Отчество" path="second_name")
          n-input(
            v-model:value="formModel.second_name"
            placeholder="Введите отчество"
          )
      n-gi
        n-form-item(label="Дата рождения" path="birthday")
          n-date-picker(
            v-model:value="formModel.birthday"
            :default-value="defaultDate"
            :default-calendar-start-time="defaultDate"
            format="dd.MM.yyyy"
            type="date"
          )
      
      n-gi(span="2")
        n-form-item(label="Клиника" path="clinic_id")
          n-select(
            v-model:value="formModel.clinic_id"
            :options="clinicOptions"
            placeholder="Выберите клинику"
          )
      
      n-gi(span="2")
        n-form-item(label="Отдел" path="department_id")
          n-select(
            v-model:value="formModel.department_id"
            :options="departmentOptions"
            placeholder="Выберите отдел"
          )

//- Диалог подтверждения удаления
n-modal(
  v-model:show="showDeleteConfirm"
  preset="dialog"
  type="error"
  title="Подтверждение удаления"
  content="Вы уверены, что хотите удалить этого сотрудника?"
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
  NCollapse,
  NCollapseItem,
  NButton,
  NTooltip,
  NIcon,
  NSpace,
  NDivider,
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
  NSwitch,
  NTag,
  NCheckbox,
  NDatePicker,
  NSelect,
  useMessage,
  NProgress,
  type FormRules,
  type FormInst,
  type SelectOption,
  type FormItemRule
} from 'naive-ui'
import { FaceCool} from '@vicons/carbon'
import { LabelWithDescription } from './label_with_description';
import { AddOutline as AddIcon, SearchOutline as SearchIcon } from '@vicons/ionicons5'
import { Diagram } from '@vicons/carbon'

import { notify_service } from '@/services/notification_service'
import { z } from 'zod'
import { uuidv7 } from 'uuidv7';
import { AddEmployeeSchema, EmployeeSchema, UpdateEmployeeSchema, type Employee } from '@/types/employees';
import { useDictionaries } from '@/composables/useDictionaries';
import { http_sevice } from '@/services/http_service/http_service';
import { DateFormat, DateTime } from '@/services/date';
import {birthday_cake_ico} from '@services/svg'
import SvgIcon from '@/components/SvgIcon.vue';
import TagWithProgress from './TagWithProgress.vue';
import SimpleStatistic from './SimpleStatistic.vue';
import StringStatistic from './StringStatistic.vue';


</script>

<script lang="ts" setup>
const formRef = ref<FormInst | null>(null)
const { statuses, statusOptions, get_status, departmentOptions, clinics, clinicOptions } = useDictionaries();
// Состояние
const employees = ref<Employee[]>([])
const searchQuery = ref('')
const showModal = ref(false)
const showDeleteConfirm = ref(false)
const editingEmployee = ref<Employee | null>(null)
const employeeToDelete = ref<Employee | null>(null)
const selected_department = ref<string | null>(null)
const selected_status = ref<string | null>(null)
const test_percentage = ref(78);
const emit = defineEmits<{
  (e: 'selected', state: Employee): void
}>()
const defaultDate = computed(() => 
{
  const date = new Date()
  date.setFullYear(date.getFullYear() - 18)
  return date.getTime()
})
const load_employees = async (dep_id?: string) =>
{
    const emp = await http_sevice.employees_service.get_employees_with_status(dep_id);
    if(emp)
        employees.value = emp
}
watch(selected_department, async (n, o) => 
{
    if(n)
    {
        await load_employees(n)
    }
    else
    {
        await load_employees()
    }
}, {immediate: true})

// Модель формы
const formModel = ref({
  first_name: '',
  second_name: '',
  surname: '',
  birthday: null as number | null,
  clinic_id: '',
  department_id: ''
})

const current_status_options = computed(() => 
{
  return statusOptions.value.filter(f=> filteredEmployees.value.map(e=> e.status_id).includes(f.value))
})

const is_birthday = (date: DateTime| null): boolean => 
{
  if(date)
  {
    const date_now = DateTime.new()
    if ((date_now.mounth == date.mounth) && (date_now.day == date.day))
    {
      return true
    }
    else
      return false
  }
  else
  {
    return false;
  }
}
const years_count = (date: DateTime| null): number => 
{
  if(date)
  {
    const date_now = DateTime.new()
    const between = date_now.year - date.year;
    if(date_now.mounth >= date.mounth && date_now.day >= date_now.day)
    {
      return between
    }
    else
    {
      return between -1
    }
  }
  else
  {
    return 0;
  }
}

// Правила валидации
const formRules: FormRules = {
  first_name: [
    {
      required: true,
      message: 'Имя обязательно',
      trigger: ['blur', 'input']
    }
  ],
  surname: [
    {
      required: true,
      message: 'Фамилия обязательна',
      trigger: ['blur', 'input']
    }
  ],
  birthday: [
    {
      trigger: ['blur', 'change'],
      validator: (rule: FormItemRule, value: Date | null) =>
      {
        return new Promise<void>((resolve, reject) =>
        {
          if(value)
          {
           const date = DateTime.parse(value);
            if(years_count(date) < 18)
            {
              reject(new Error("Возраст не может быть меньше 18 лет"))
            }
            else resolve();
          }
          else reject(new Error("Дата рождения обязательна"))
         
        })
        
      }
    },
  ],
  clinic_id: [
    {
      required: true,
      message: 'Клиника обязательна',
      trigger: ['blur', 'change']
    }
  ],
  department_id: [
    {
      required: true,
      message: 'Отдел обязателен',
      trigger: ['blur', 'change']
    }
  ]
}

const filteredEmployees = computed(() => 
{
  let filtered = employees.value

  // Поиск по ФИО
  if (searchQuery.value) 
  {
    const query = searchQuery.value.toLowerCase()
    filtered = filtered.filter(employee =>
      `${employee.surname} ${employee.first_name} ${employee.second_name}`
        .toLowerCase()
        .includes(query)
    )
  }
  if(selected_status.value)
  {
    filtered = filtered.filter(emp => emp.status_id != undefined && emp.status_id == selected_status.value)
  }

  return filtered
})

// Вспомогательные функции
const fullName = (employee: Employee) => 
{
  return `${employee.surname} ${employee.first_name} ${employee.second_name}`
}

const getInitials = (employee: Employee) => 
{
  return `${employee.first_name[0]}${employee.surname[0]}`.toUpperCase()
}

const employeeDescription = (employee: Employee) => 
{
  const department = departmentOptions.value.find(d => d.value === employee.department_id)
  return `${department?.label || 'Неизвестно'}`
}
const select_statuses_handle = (emp: Employee) =>
{
  emit('selected', emp);
}

const formatDate = (date: DateTime| null): string| null => 
{
  return date ? date.to_string(DateFormat.CalendarFormat) : ""
}

// Открытие модального окна для добавления
const openAddModal = () => 
{
  editingEmployee.value = null
  formModel.value = {
    first_name: '',
    second_name: '',
    surname: '',
    birthday: null,
    clinic_id: '',
    department_id: ''
  }
  showModal.value = true
}

// Открытие модального окна для редактирования
const openEditModal = (employee: Employee) => 
{
  editingEmployee.value = employee
  formModel.value = 
  {
    first_name: employee.first_name,
    second_name: employee.second_name,
    surname: employee.surname,
    birthday: employee.birthday?.as_date().getTime() ?? null,
    clinic_id: employee.clinic_id,
    department_id: employee.department_id
  }
  showModal.value = true
}

// Подготовка к удалению
const confirmDelete = (employee: Employee) => 
{
  employeeToDelete.value = employee
  showDeleteConfirm.value = true
}

// Удаление сотрудника
const handleDelete = async () => 
{
  if (employeeToDelete.value) 
  {
    const deleted = await http_sevice.employees_service.delete(employeeToDelete.value.id);
    if(deleted)
    {
      employees.value = employees.value.filter(emp => emp.id !== employeeToDelete.value!.id)
      notify_service.notify_success('Сотрудник удален', fullName(employeeToDelete.value as Employee));
      employeeToDelete.value = null;
      showDeleteConfirm.value = false
    }
    
  }
  
}

// Сохранение сотрудника
const handleSave = async (): Promise<boolean> => 
{
  const errors = await formRef.value?.validate();
  console.log(errors);
  if (errors?.warnings) 
  {
    notify_service.notify_error('Пожалуйста, исправьте ошибки в форме', '')
    //showModal.value = true;
    console.error("Ошибки при валидации данных ", errors.warnings);
    return false
  }
  else
  {
    if(editingEmployee.value)
    {
      //валидация данных по схеме
      const update_employee = UpdateEmployeeSchema.safeParse(
      {
        id: editingEmployee.value.id,
        ...formModel.value,
        birthday: formModel.value.birthday!,
      })
      if (!update_employee.success) 
      {
        notify_service.notify_error('Неверные данные сотрудника', '')
        console.error(update_employee.error)
        return false
      }
      //обновление сотрудника на бэке
      const updated_employee = await http_sevice.employees_service.edit(update_employee.data);
      if(updated_employee)
      {
        const index = employees.value.findIndex(emp => emp.id === updated_employee.id)
        if (index !== -1) 
        {
          employees.value[index] = updated_employee
        }
        notify_service.notify_success('Сотрудник обновлен', fullName(updated_employee))
        showModal.value = false;
        resetForm();
        return true;
      }
      else return false;
    }
    else
    {
      //валидация данных по схеме
      const add_employee = AddEmployeeSchema.safeParse(
      {
        ...formModel.value,
        birthday: formModel.value.birthday!,
      })
      if (!add_employee.success) 
      {
        notify_service.notify_error('Неверные данные сотрудника', '')
        console.error(add_employee.error);
        return false;
      }
      //обновление сотрудника на бэке
      const added_employee = await http_sevice.employees_service.add(add_employee.data);
      if(added_employee)
      {
        employees.value.push(added_employee);
        notify_service.notify_success('Сотрудник добавлен', fullName(added_employee))
        showModal.value = false;
        resetForm();
        return true;
      }
      else return false;
    }
  }
}

// Отмена редактирования
const handleCancel = () => 
{
  resetForm()
  return true;
}

// Сброс формы
const resetForm = () => 
{
  formModel.value = 
  {
    first_name: '',
    second_name: '',
    surname: '',
    birthday: null,
    clinic_id: '',
    department_id: ''
  }
  editingEmployee.value = null
}

const update_status = (employee_id: string, state_id: string, status_id: string, date_from: DateTime, date_to: DateTime) =>
{
  const index = employees.value.findIndex(emp => emp.id === employee_id)
  if (index !== -1) 
  {
    const date_now = DateTime.new();
    const date_between = date_now.between(date_from, date_to);
    //при обновлении текущего статуса
    //проверка что дата статуса перекрывает текущую дату, 
    //обновляем только если текущий статус активен или стал активен после редактирования
    if(state_id == employees.value[index].state_id)
    {
      if(date_between)
      {
        employees.value[index].status_id = status_id;
        employees.value[index].state_start_date = date_from;
        employees.value[index].state_end_date = date_to;
      }
      else
      {
        delete_status(state_id);
      }
      
    }
    else
    {
      if(date_between)
      {
        employees.value[index].status_id = status_id;
        employees.value[index].state_id = state_id;
        employees.value[index].state_start_date = date_from;
        employees.value[index].state_end_date = date_to;
      }
    }
  }
}
const delete_status = (state_id: string) =>
{
  const index = employees.value.findIndex(emp => emp.state_id === state_id)
  if (index !== -1) 
  {
    employees.value[index].status_id = undefined;
    employees.value[index].state_id = undefined;
    employees.value[index].state_start_date = null;
    employees.value[index].state_end_date = null;
  }
}

defineExpose({
  update_status,
  delete_status
})
</script>

<style lang="scss" scoped>
.employee-manager 
{
  max-width: 900px;
  margin: 0 auto;
  min-width: 800px;
}

.n-list 
{
  max-height: calc(100vh - 344px);
  overflow-y: auto;
}

.n-list-item 
{
  padding: 16px;
}

.n-avatar 
{
  font-weight: bold;
  background-color: #1890ff;
  color: white;
}

.switch-place 
{
  display: flex;
  flex-direction: row;
  margin-top: 10px;
  gap: 10px;
  align-items: center;
}
.birthday
{
  display: flex;
  gap: 5px;
}
.filter-panel
{
  display: flex;
  flex-direction: row;
  gap: 10px;
}
.pg
{
  position:relative;
  left: 10px;
  width: 100%;
}
.edit-button
{
  margin-left: 10px;
}
.statistic-collapse
{
  width: 100%;
}
</style>