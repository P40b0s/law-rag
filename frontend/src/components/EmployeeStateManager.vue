<template lang="pug">
n-card.employee-state-manager(:title="title" v-if="props.employee != null")
  template(#header-extra)
    n-button.edit-button(type="primary" @click="openAddModal")
      template(#icon)
        n-icon: add-icon
      | Добавить

  n-space(vertical :size="20")
    //- Фильтры
    .filter-panel
        n-select(v-model:value="selected_status"
            clearable
            :options="statusOptions"
            placeholder="Фильтр по состоянию")
        n-date-picker(
            v-model:value="search_month"
            type="month"
            clearable
            placeholder="Месяц")
    n-checkbox(v-model:checked="showDiseaseOnly") Болен
    

    //- Список статусов
    loader(v-if="is_loading || loading")
    n-list(bordered v-else)
      n-list-item(v-for="state in filteredStates" :key="state.id")
        template(#suffix)
          n-space
            n-button(size="small" @click="openEditModal(state)") Редактировать
            n-button(size="small" type="error" @click="confirmDelete(state)") Удалить
        
        n-thing(:title="getStatusTitle(state.status_id)" :description="state.note ? state.note: ''")
          template(#avatar v-if="get_status(state.status_id).logo")
            svg-icon-native(:size="25" :svg="get_status(state.status_id).logo")
          
          template(#footer)
            n-space.footer-panel(size="small")
              div С
              n-tag(size="medium" type="success") {{ formatDate(state.start_date) }}
              div по
              n-tag(size="medium" type="warning") {{ formatDate(state.end_date) }}
              n-tag(size="medium" type="info") {{ days_count(state.start_date, state.end_date) }} д.
              

    //- Пустое состояние
    n-empty(
      v-if="filteredStates.length === 0"
      description="Состояния не найдены"
    )
      template(#extra)
        n-button(size="small" @click="openAddModal") Добавить

//- Модальное окно добавления/редактирования
n-modal(
  v-model:show="showModal"
  :title="editingState ? 'Редактировать состояние' : 'Добавить состояние'"
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
    n-grid(:cols="3" :x-gap="24")
      n-gi(span="3")
        n-form-item(label="Статус" path="status_id")
          n-select(
            v-model:value="formModel.status_id"
            :options="statusOptions"
            placeholder="Выберите статус"
            filterable
          )
      
      n-gi.date-panel(span="3" v-if="editingState == null")
        n-checkbox(v-model:checked="multidata_selector") 
        label-with-description( style="flex-basis: 27%;" name="Несколько дат", description="Возможность выбрать сразу несколько дат для выбранного статуса, возможно только при добавлении нового статуса")
        .date-panel(v-if="multidata_selector")
          n-form-item(label="Выберите несколько дат" path="dates_multiple")
            vue-date-picker(
              v-model="formModel.dates_multiple"
              multi-dates
              :format="format_multiple"
              multi-calendars
              dark
              :disabled-dates="disabled_dates[0]"
              :markers="disabled_dates[1]"
              :enable-time-picker="false"
              :state="date_multiple_error_state"
              locale='ru' cancelText="отмена" selectText="подтвердить")
        .date-panel(v-else)
          n-form-item(label="Выберите период" path="dates_range")
            vue-date-picker(
              v-model="formModel.dates_range"
              multi-calendars
              dark
              range
              locale='ru' cancelText="отмена" selectText="подтвердить"
              :format="format_range"
              :preview-format="format_range_preview"
              :disabled-dates="disabled_dates[0]"
              :markers="disabled_dates[1]"
              :state="date_range_error_state"
              :enable-time-picker="false"
              @internal-model-change="handle_date_selection")
        
          n-form-item(label="Количество дней" style="flex-basis: 37%" v-if="formModel.dates_range")
            n-input-number(
              v-model:value="formModel.days_count"
              :min="1"
              @update:value="handle_days_count_change"
            )

      n-gi.date-panel(span="3" v-else)
        n-form-item(label="Выберите период" path="dates_range" style="flex-basis: 73%")
          vue-date-picker(
            v-model="formModel.dates_range"
            multi-calendars
            dark
            range
            locale='ru' cancelText="отмена" selectText="подтвердить"
            :format="format_range"
            :preview-format="format_range_preview"
            :disabled-dates="disabled_dates[0]"
            :markers="disabled_dates[1]"
            :state="date_range_error_state"
            :enable-time-picker="false"
            @internal-model-change="handle_date_selection")

        n-form-item(label="Количество дней" style="flex-basis: 27%" v-if="formModel.dates_range")
          n-input-number(
              v-model:value="formModel.days_count"
              :min="1"
              @update:value="handle_days_count_change"
            )

      n-gi(span="3")
        n-form-item(label="Примечание" path="note")
          n-input(
            v-model:value="formModel.note"
            type="textarea"
            placeholder="Дополнительная информация"
            :autosize="{ minRows: 2, maxRows: 4 }"
          )

//- Диалог подтверждения удаления
n-modal(
  v-model:show="showDeleteConfirm"
  preset="dialog"
  type="error"
  title="Подтверждение удаления"
  :content="`Вы уверены, что хотите удалить статус ${stateToDelete ? getStatusTitle(stateToDelete.status_id) : ''}?`"
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
  NSwitch,
  NSelect,
  NCheckbox,
  NInputNumber,
  NTabs,
  NTabPane,
  type FormRules,
  type FormInst,
  type SelectOption,
  type FormItemRule,
  type DatePickerProps
} from 'naive-ui'

import { AddOutline as AddIcon, SearchOutline as SearchIcon, ArrowForwardOutline} from '@vicons/ionicons5'
import { disease_ico, disease_red_ico, palm_ico } from '@/services/svg'
import { notify_service } from '@/services/notification_service'
import { http_sevice } from '@/services/http_service/http_service'
import { useDictionaries } from '@/composables/useDictionaries'
import SvgIcon from '@/components/SvgIcon.vue'
import { DateFormat, DateTime } from '@/services/date'
import type { EmployeeStatus } from '@/types/employee_status'
import { type EmployeeState, type Employee, type EmployeeStates, EmployeeNewStateSchema, type EmployeeNewState } from '@/types/employees'
import { insert_into_sorted_array } from '@/services/helpers'
import { LabelWithDescription } from './label_with_description';
import Loader from './Loader.vue';
import SvgIconNative from './SvgIconNative.vue';
interface Markers 
{
    date: Date | string;
    type?: 'dot' | 'line';
    tooltip?: { text: string; color?: string;}[];
    color?: string;
    // el is a HTML element of a calendar cell
    customPosition?: (el: HTMLElement) => Record<string, string | number>;
}
//TODO заготовка для группового удаления состояний, на потом!
type EmployeeStateAndSelected = EmployeeState & {selected?: boolean};
</script>

<script lang="ts" setup>
const props = defineProps<{
  employee: Employee | null
}>()

const emit = defineEmits<{
  (e: 'saved', state: EmployeeState): void
  (e: 'deleted', state_id: string): void
}>()

const formRef = ref<FormInst | null>(null)
const { statuses, statusOptions, get_status, loading} = useDictionaries()

// Состояние
const states = ref<EmployeeStateAndSelected[] | null>()
//const searchQuery = ref('')
const showModal = ref(false)
const showDeleteConfirm = ref(false)
const selected_status = ref<string | null>(null)
const search_month = ref<number | null>(null)
const editingState = ref<EmployeeState | null>(null)
const stateToDelete = ref<EmployeeState | null>(null)
const showDiseaseOnly = ref(false);
const days_count_visible = ref(false);
const multidata_selector = ref(false);
const is_loading = ref(false)
// Модель формы
const formModel = ref({
  status_id: '',
  days_count: null as number | null,
  dates_range: null as Date[] | null,
  dates_multiple: null as Date[] | null,
  note: ''
})
const date_range_error_state = computed(() =>
{
  return formModel.value.dates_range != null
})
const date_multiple_error_state = computed(() =>
{
  return formModel.value.dates_multiple != null
})
const days_count_visible_state = computed(() => days_count_visible && formModel.value.dates_range)

// Правила валидации
const formRules: FormRules = {
  status_id: [
    {
      required: true,
      message: 'Статус обязателен',
      trigger: ['blur', 'change']
    }
  ],
  start_date: [
    {
      trigger: ['blur', 'change'],
      validator: (rule: FormItemRule, value: Date | null) =>
      {
        return new Promise<void>((resolve, reject) =>
        {
          if(value)
          {
            resolve();
          }
          else reject(new Error("Дата окончания обязательна!"))
         
        })
      }
    }
  ],
  dates_multiple: [
    {
      trigger: ['blur', 'change'],
      validator: (rule: FormItemRule, value: number | null) =>
      {
        return new Promise<void>((resolve, reject) =>
        {
          if(value)
          {
            if(formModel.value.dates_multiple && formModel.value.dates_multiple.length > 0)
              return resolve();
            else
            {
              reject(new Error("Необходимо выбрать хотя бы одну дату!"))
              // if(value < formModel.value.start_date)
              // {
              //   reject(new Error("Дата окончания должна быть больше даты начала!"))
              // }
              // else resolve();
            }
          }
          else reject(new Error("Необходимо выбрать хотя бы одну дату!"))
         
        })
      }
    }
  ],
    dates_range: [
    {
      trigger: ['blur', 'change'],
      validator: (rule: FormItemRule, value: Date[] | null) =>
      {
        return new Promise<void>((resolve, reject) =>
        {
          if(value)
          {
            if(formModel.value.dates_range && formModel.value.dates_range.length > 0)
              return resolve();
            else
            {
              reject(new Error("Необходимо выбрать период!"))
              // if(value < formModel.value.start_date)
              // {
              //   reject(new Error("Дата окончания должна быть больше даты начала!"))
              // }
              // else resolve();
            }
          }
          else reject(new Error("Необходимо выбрать период!"))
         
        })
      }
    }
  ]
}

// Computed свойства
const title = computed(() => `Состояния: ${props.employee?.surname} ${props.employee?.first_name} ${props.employee?.second_name}`)
//фильрация
const filteredStates = computed(() => 
{
  let filtered = states.value
  if(filtered)
  {
    // Фильтр по типу статуса
    if (selected_status.value) 
    {
        filtered = filtered.filter(state => 
        {
            const status = get_status(state.status_id)
            return status?.id == selected_status.value
        })
    }
    //фильтр по только заболевшим
    if(showDiseaseOnly.value)
    {
        filtered = filtered.filter(state => 
        {
          const status = get_status(state.status_id)
          return status?.is_disease
        })
    }

    // Фильтр по дате
    if (search_month.value) 
    {
        const searched_month = DateTime.parse(search_month.value);
        filtered = filtered?.filter(state => 
        {
            if(state.start_date && state.end_date)
            {
                if((state.start_date?.year == searched_month.year && state.start_date.mounth == searched_month.mounth)
                || (state.end_date?.year == searched_month.year && state.end_date.mounth == searched_month.mounth))
                {
                  return true;
                }
                else return false;
            }
            else return false;
        })
    }
    return filtered
  }
  else return []
})

// Вспомогательные функции
const getStatusTitle = (statusId: string) => 
{
  return get_status(statusId)?.status || 'Неизвестный статус'
}

const getStatusAvatarType = (statusId: string): 'error' | 'warning' | 'info' | 'success' => 
{
  const status = get_status(statusId)
  if (!status) return 'info'
  return status.is_disease ? (status.tracing ? 'error' : 'warning') :
         status.on_work_place ? 'success' : 'info'
}

const getStatusIcon = (statusId: string) => 
{
  const status = get_status(statusId)
  if (!status) return null
  return status.is_disease ? (status.tracing ? disease_red_ico : disease_ico) : palm_ico
}


const getTagType = (statusName: string): 'default' | 'error' | 'warning' | 'success' | 'info' => 
{
  const status = Array.from(statuses.value.values()).find(s => s.status === statusName)
  if (!status) return 'default'
  return status.is_disease ? (status.tracing ? 'error' : 'warning') :
         status.on_work_place ? 'success' : 'info'
}


const disabled_dates = computed<[Date[], Markers[]]>(() => 
{
  let dates: Date[] = [];
  let markers: Markers[] = [];
  if(states.value)
  {
    let current_states = states.value;
    if(editingState.value)
    {
      current_states = states.value.filter(f=>f.id != editingState.value?.id)
    }
    for(let i = 0; i< current_states.length; i++)
    {
      const start_date = current_states[i].start_date;
      const end_date = current_states[i].end_date;
      if(start_date && end_date)
      {
        const status = get_status(current_states[i].status_id);
        const dis_dates = DateTime.between_dates_array(start_date, end_date);
        if(status)
        {
          const color = status.color;
          dis_dates.forEach(f=>markers.push({
            date: f,
            type: 'dot',
            tooltip:
            [{
              text: status.status,
              color: color
            }],
            color: color,
          }))
        }
        
        dates = dates.concat(dis_dates);
      }
    }
  }
  return [dates, markers];
})

//происходит перед выбором диапазона дат
const handle_date_selection = (newRange: Date[]): boolean => 
{
  if (!newRange || newRange.length !== 2) return false;
  const selected_range = DateTime.between_dates_array(DateTime.parse(newRange[0]), DateTime.parse(newRange[1])).map(m=>m.getTime());
  for(let i = 0; i< selected_range.length; i++)
  {
    if(disabled_dates.value[0].map(m=>m.getTime()).includes(selected_range[i]))
    {
      console.error("Зафиксировано пересечение")
      notify_service.notify_error('Выбранный диапазон содержит недоступные даты', '')
      formRef.value?.validate();
      formModel.value.dates_range = null;
      days_count_visible.value = false;
      return false;
    }
  }
  days_count_visible.value = true;
  return true;
};

// function validate_date_range(selectedRange: Date[], disabledDates: Date[]) 
// {
//   if (!selectedRange || selectedRange.length !== 2) return true;
  
//   const [start, end] = selectedRange;
//   if (!start || !end) return true;
  
//   // Конвертируем в timestamp для быстрого сравнения
//   const startTime = start.getTime();
//   const endTime = end.getTime();
//   const disabledTimestamps = new Set(disabledDates.map(d => d.getTime()));
  
//   // Быстрая проверка граничных значений
//   const minDisabled = Math.min(...disabledTimestamps);
//   const maxDisabled = Math.max(...disabledTimestamps);
  
//   if (endTime < minDisabled || startTime > maxDisabled) {
//     return true; // Нет пересечения
//   }
  
//   // Проверяем каждую disabled дату
//   for (const timestamp of disabledTimestamps) {
//     if (timestamp >= startTime && timestamp <= endTime) {
//       return false; // Найдено пересечение
//     }
//   }
  
//   return true; // Нет пересечения
// }


const format_multiple = (date: string | Date | Date[]) => 
{
  const dates = date as Date[];
  return dates.map(d=>
    {
      const day = d.getDate().toString().padStart(2, "0");
      const month = (d.getMonth() + 1).toString().padStart(2, "0");
      const year = d.getFullYear();
      return `${day}.${month}.${year}`;
    }
  ).join(", ")
}

const format_range = (date: Date[]) => 
{
  const dates = date as Date[];
  return dates.map(d=>
    {
      const day = d.getDate().toString().padStart(2, "0");
      const month = (d.getMonth() + 1).toString().padStart(2, "0");
      const year = d.getFullYear();
      return `${day}.${month}.${year}`;
    }
  ).join(" - ")
}

const format_range_preview = (date: Date[]) => 
{
  if(date[0] && date[1])
  {
    const dc = DateTime.parse(date[0]).days_between(DateTime.parse(date[1]));
    formModel.value.days_count = dc;
    return `Количество дней: ${dc}`;
  }
}


const formatDate = (date: DateTime| null): string| null => 
{
  return date ? date.to_string(DateFormat.CalendarFormat) : ""
}
const days_count = (start_date: DateTime| null, end_date: DateTime| null) =>
{
  if(start_date && end_date)
  {
    return start_date.days_between(end_date)
  }
  else return 0
}
const handle_days_count_change = (n: number | null) =>
{
  if(formModel.value && formModel.value.dates_range && n)
  {
    const start_date = new Date(formModel.value.dates_range[0]);
    formModel.value.dates_range[1] = new Date((start_date.setDate(start_date.getDate() + (n-1)))); 
  }
}
// const handle_days_count_change_old = (n: number | null) =>
// {
//   if(formModel.value && formModel.value.start_date && n)
//   {
//     const start_date = new Date(formModel.value.start_date);
//     formModel.value.end_date = (start_date.setDate(start_date.getDate() + (n-1))); 
//   }
// }
// const handle_state_start_date_change = (date: number) =>
// {
//   if(formModel.value && formModel.value.start_date && formModel.value.days_count)
//   {
//     const start_date = new Date(date);
//     formModel.value.end_date = (start_date.setDate(start_date.getDate() + (formModel.value.days_count-1))); 
//   }
// }

// const handle_state_end_date_change = (date: number) =>
// {
//   if(formModel.value && formModel.value.start_date)
//   {
//     const count = days_count(DateTime.parse(formModel.value.start_date), DateTime.parse(date));
//     formModel.value.days_count = count; 
//   }
// }

// Методы
const openAddModal = () => 
{
  editingState.value = null
  formModel.value = 
  {
    status_id: '',
    days_count: 0,
    dates_range: null,
    dates_multiple: null,
    note: ''
  }
  showModal.value = true
}

const openEditModal = (state: EmployeeState) => 
{
  editingState.value = state
  formModel.value = 
  {
    status_id: state.status_id,
    days_count: days_count(state.start_date, state.end_date),
    dates_range: get_dates_range(state.start_date, state.end_date),
    dates_multiple: null,
    note: state.note || ''
  }
  showModal.value = true
}
const get_dates_range = (date_1: DateTime|null, date_2: DateTime | null): Date[]|null =>
{
  if(date_1 == null || date_2 == null)
    return null;
  return [date_1.as_date(), date_2.as_date()]
}

const confirmDelete = (state: EmployeeState) => 
{
  stateToDelete.value = state
  showDeleteConfirm.value = true
}

const handleDelete = async () => 
{
  if (!stateToDelete.value) return false
  const del = await http_sevice.employee_states_service.delete(stateToDelete.value.id);
  if(del)
  {
    states.value = states.value?.filter(s => s.id !== stateToDelete.value?.id)
    emit('deleted', stateToDelete.value.id);
    stateToDelete.value = null;
    notify_service.notify_success('Статус успешно удален', '')
    showDeleteConfirm.value = false
    return true;
  }
  else
  {
    notify_service.notify_error('Ошибка при удалении статуса', '');
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
      const range = formModel.value.dates_range as Date[];
      const saved = 
      {
        ...formModel.value,
        id: editingState.value.id,
        employee_id: editingState.value.employee_id,
        start_date: DateTime.parse(range[0]),
        end_date: DateTime.parse(range[1]),
        
      }
      const result = await http_sevice.employee_states_service.edit(saved);
      if(result && states.value)
      {
        const index = states.value.findIndex(s => s.id === result.id)
        if (index !== -1) 
        {
          states.value[index] = result;
          emit('saved', result);
        }
        notify_service.notify_success('Статус обновлен', '');
        showModal.value = false
        resetForm()
        return true
      }
      else
      {
        notify_service.notify_error('Ошибка обновления статуса', '');
        return false;
      }
    }
    else
    {
      if(multidata_selector.value && formModel.value.dates_multiple)
      {
        // const added = formModel.value.dates_multiple.map(d=>
        // {
        //   return EmployeeNewStateSchema.parse(
        //   {
        //     ...formModel.value,
        //     employee_id: props.employee?.id as string,
        //     start_date: DateTime.parse(d),
        //     end_date: DateTime.parse(d),
        //   })
        // })
        const added = formModel.value.dates_multiple.map(d=>
        {
          return {
            ...formModel.value,
            employee_id: props.employee?.id as string,
            start_date: DateTime.parse(d),
            end_date: DateTime.parse(d),
          }
        })
        const result = await http_sevice.employee_states_service.add_multiple(added);
        if(result)
        {
          const added_result = result.map(r=> added_sub(r));
          if(added_result.every(e=>e))
          {
            notify_service.notify_success('Статусы добавлены', '');
            showModal.value = false
            resetForm()
            return true;
          }
          else
          {
            notify_service.notify_error('Ошибка добавления статусов', '');
            return false;
          }
        }
      }
      else
      {
        const range = formModel.value.dates_range as Date[];
        // const added = await EmployeeNewStateSchema.safeParseAsync(
        // {
        //   ...formModel.value,
        //   status_id: props.employee?.status_id,
        //   employee_id: props.employee?.id as string,
        //   start_date: DateTime.parse(new Date(range[0])),
        //   end_date: DateTime.parse(new Date(range[1])),
        // })
        const added = 
        {
          ...formModel.value,
          employee_id: props.employee?.id as string,
          start_date: DateTime.parse(new Date(range[0])),
          end_date: DateTime.parse(new Date(range[1])),
        }
       
        const result = await http_sevice.employee_states_service.add(added);
        const add_result = added_sub(result);
        if(add_result)
        {
          notify_service.notify_success('Статус добавлен', '');
          showModal.value = false
          resetForm()
        }
        return add_result;
      }
    }
  }
}

const added_sub = (result: EmployeeState| undefined) =>
{
  if(result && states.value)
  {
    states.value = insert_into_sorted_array(states.value, result, employees_states_compare);
    //states.value.push(result);
    states.value.sort((a, b) => employees_states_compare(a, b));
    emit('saved', result);
    return true
  }
  else
  {
    return false;
  }
}


const employees_states_compare = (a: EmployeeState, b: EmployeeState): number => 
{
  if (a.start_date == null && b.start_date == null) return 0;
  if (a.start_date == null) return -1;
  if (b.start_date == null) return 1; 
  
  return b.start_date.as_date().getTime() - a.start_date.as_date().getTime();

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
    status_id: '',
    days_count: null,
    dates_range: null,
    dates_multiple: null,
    note: ''
  }
  editingState.value = null
}

// Загрузка данных при изменении employeeId
watch(() => props.employee, async (new_employee) => 
{
    if(new_employee)
    {
        const employee_states = await http_sevice.employee_states_service.get_states(new_employee.id);
        if(employee_states)
            states.value = employee_states
    }  
}, { immediate: true })

type DatePickerThemeOverrides = NonNullable<DatePickerProps['themeOverrides']>
const date_picker_theme_overrides: DatePickerThemeOverrides = {
    //itemTextColorDisabled: 'rgba(24, 127, 231, 1)',
    
  }
</script>

<style lang="scss">
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
.date-panel
{
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 10px;
}

.n-date-panel .n-date-panel-dates .n-date-panel-date.n-date-panel-date--disabled {
  color: #ff4d4f !important;
  opacity: 0.9;
  background-color: #77392f57 !important;
}

.dp__cell_disabled {
	color:  #ff4d4f;
	cursor: not-allowed;
}

/* Плавное скольжение с разными timing functions */
.slide-enter-active {
  transition: all 0.5s cubic-bezier(0.25, 0.46, 0.45, 0.94);
  position: absolute;
  width: 100%;
}

.slide-leave-active {
  transition: all 0.4s cubic-bezier(0.55, 0.085, 0.68, 0.53);
  position: absolute;
  width: 100%;
}

.slide-enter-from {
  opacity: 0;
  transform: translateX(-100%) scale(0.95);
}

.slide-enter-to {
  opacity: 1;
  transform: translateX(0) scale(1);
}

.slide-leave-from {
  opacity: 1;
  transform: translateX(0) scale(1);
}

.slide-leave-to {
  opacity: 0;
  transform: translateX(100%) scale(0.95);
}
</style>