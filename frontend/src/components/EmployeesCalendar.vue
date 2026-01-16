<template lang="pug">
n-card
  .calendar-matrix
    .calendar-header
      n-button(secondary @click="prevMonth" type="success")
        template(#icon)
          n-icon
            ArrowBack
        | Предыдущий месяц
      h2 {{ currentMonthYear }}
      n-button(secondary @click="nextMonth" type="success")
        | Следующий месяц
        template(#icon)
          n-icon
            ArrowForward
  .filter-panel
    n-select(v-model:value="selected_department"
            clearable
            :options="departmentOptions"
            placeholder="Фильтр по отделу")
    n-select(v-model:value="selected_status"
          clearable
          :options="statusOptions"
          placeholder="Фильтр по состоянию")

  .matrix-container
    .people-column
      .person-header Люди
      .person-row(
        v-for="person in filtered_employees"
        :key="person.employee.id"
        :class="{ 'current-person': currentPerson?.employee.id === person.employee.id }"
        @click="selectPerson(person)"
      )
        .person-info
          .person-name {{ person.employee.surname }} {{ person.employee.first_name[0]}}.{{person.employee.second_name[0]}}.

    .dates-container
      .dates-header
        n-grid(
          :cols="daysInMonth"
          :x-gap="0"
          :y-gap="0"
          responsive="screen"
        )
          n-gi(
            v-for="day in daysInMonth"
            :key="day"
            :class="getDayHeaderClass(day)"
          )
            .day-header-cell
              .day-number {{ day }}
              .day-name {{ getDayName(day) }}

      .dates-grid
        .person-row(
          v-for="person in filtered_employees"
          :key="person.id"
        )
          n-grid(
            :cols="daysInMonth"
            :x-gap="0"
            :y-gap="0"
            responsive="screen"
          )
            
            n-gi(
              v-for="day in daysInMonth"
              :key="day"
              :class="getDayCellClass(person, day)"
              @click="handleCellClick(person, day)"
              @mouseenter="handleCellHover(person, day, $event)"
              @mouseleave="handleCellLeave"
            )
              .day-cell
                n-tooltip(v-if="hasPeriodOnDay(person, day)" trigger="hover")
                  template(#trigger)
                    .period-marker(:style="{ backgroundColor: getDayCellColor(person, day) }")
                      svg-icon-native(:svg="getDayCellLogo(person, day)" :size="25")
                  div {{ getTooltipContent(person, day) }}

  n-modal(v-model:show="showCellModal" preset="card" title="Информация о ячейке")
    .cell-modal-content(v-if="selectedCell")
      n-h4 {{ selectedCell.person.name }}
      n-text(depth="3") Дата: {{ formatModalDate(selectedCell.day) }}
      .cell-periods(v-if="selectedCell.period")
        n-tag(:color="{ color: selectedCell.person.color }") 
          | Период: {{ selectedCell.period.startDate }} - {{ selectedCell.period.endDate }}
</template>
<script setup lang="ts">
import { ref, computed, reactive, h, watch } from 'vue'
import { 
  NConfigProvider, 
  NButton, 
  NIcon, 
  NGrid, 
  NGi, 
  NTag, 
  NSpace, 
  NH3, 
  NDivider, 
  NTooltip,
  NModal,
  NH4,
  NText,
  NCard,
  NSelect,
  type GlobalTheme 
} from 'naive-ui'
import { ArrowBack, ArrowForward } from '@vicons/ionicons5'
import { onMounted } from 'vue'
import { useDictionaries } from '@/composables/useDictionaries';
import { type EmployeeState, type CalendarEmployee, type Employee } from '@/types/employees';
import { http_sevice } from '@/services/http_service/http_service';
import { DateFormat, DateTime } from '@/services/date';
import { ColorHelper } from '@/services/helpers';
import SvgIconNative from './SvgIconNative.vue';


const { statuses, statusOptions, get_status, departmentOptions, clinics, clinicOptions } = useDictionaries();
// Типы

interface  SelectedCell {
  person: CalendarEmployee
  day: number
  period: EmployeeState | null
}

// Emits
const emit = defineEmits<{
  cellClick: [person: CalendarEmployee, date: Date, period: EmployeeState | null]
  personSelect: [person: CalendarEmployee]
}>()
// Data
const selected_department = ref<string | null>(null)
const selected_status = ref<string | null>(null)
const employees = ref<CalendarEmployee[]>([])
const currentDate = ref(new Date())
const currentPerson = ref<CalendarEmployee | null>(null)
const showCellModal = ref(false)
const selectedCell = ref<SelectedCell | null>(null)
const hoveredColumn = ref<number | null>(null)
const filtered_employees = computed(() => 
{
  let status_exists = [];
  if(selected_status.value)
    for(const e of employees.value)
    {
      const status = e.statuses.filter(f=> f.status_id == selected_status.value);
      if(status.length > 0)
        status_exists.push({
          employee: e.employee,
          statuses: status
      });
    }
  if(selected_department.value)
  {
    if(status_exists.length > 0)
    {
      status_exists = status_exists.filter(f=>f.employee.department == selected_department.value);
    }
    else
    {
      status_exists = employees.value.filter(f=>f.employee.department == selected_department.value);
    }
  }
  if(selected_status.value == null && selected_department.value == null)
    status_exists = employees.value;
  return status_exists;
})

// const current_status_options = computed(() => 
// {
//   return statusOptions.value.filter(f=> filtered_employees.value.filter(fe=>fe.statuses.map(m=>m.status_id).includes(f.value)));
// })

const load_employees = async () =>
{
  const year = currentDate.value.getFullYear();
  const month = currentDate.value.getMonth() +1;
  const peoples = await http_sevice.employees_service.get_calendar_employees(year, month, selected_department.value ?? undefined);
  if(peoples)
      employees.value = peoples
}
watch(selected_department, async (n, o) => 
{
  await load_employees()
}, {immediate: true})

watch(currentDate, async (n, o) => 
{
  await load_employees()
    
}, {immediate: true})








const currentMonthYear = computed(() => {
  return currentDate.value.toLocaleDateString('ru-RU', {
    month: 'long',
    year: 'numeric'
  })
})

const daysInMonth = computed(() => {
  const year = currentDate.value.getFullYear()
  const month = currentDate.value.getMonth()
  return new Date(year, month + 1, 0).getDate()
})

const cellSize = computed(() => {
  // Адаптивный размер ячейки в зависимости от количества дней
  const baseSize = 60
  const maxCells = 31
  const scale = Math.min(1, maxCells / daysInMonth.value)
  return Math.max(40, baseSize * scale)
})

// Methods
const prevMonth = (): void => {
  currentDate.value = new Date(
    currentDate.value.getFullYear(),
    currentDate.value.getMonth() - 1,
    1
  )
}

const nextMonth = (): void => {
  currentDate.value = new Date(
    currentDate.value.getFullYear(),
    currentDate.value.getMonth() + 1,
    1
  )
}

const getDayName = (day: number): string => {
  const date = new Date(
    currentDate.value.getFullYear(),
    currentDate.value.getMonth(),
    day
  )
  const dayNames = ['Вс', 'Пн', 'Вт', 'Ср', 'Чт', 'Пт', 'Сб']
  return dayNames[date.getDay()]
}

const isWeekend = (day: number): boolean => {
  const date = new Date(
    currentDate.value.getFullYear(),
    currentDate.value.getMonth(),
    day
  )
  const dayOfWeek = date.getDay()
  return dayOfWeek === 0 || dayOfWeek === 6
}



const formatDateKey = (day: number): Date => 
{
  const year = currentDate.value.getFullYear()
  const month = currentDate.value.getMonth() + 1
  return new Date(`${year}-${month.toString().padStart(2, '0')}-${day.toString().padStart(2, '0')}`)
}

const isDateInPeriod = (date: Date, period: EmployeeState): boolean => 
{
  date.setHours(0, 0, 0, 0);
  if(period.start_date && period.end_date)
  {
    return (date >= period.start_date?.as_date() && date <= period.end_date?.as_date()) 
    || ((period.start_date?.as_date() == period.end_date?.as_date()) && period.start_date?.as_date() == date)
  }
  else
  {
    return false
  }
  
}

const hasPeriodOnDay = (person: CalendarEmployee, day: number): boolean => 
{
  const dateKey = formatDateKey(day)
  return person.statuses.some(period => isDateInPeriod(dateKey, period))
}
const hasPeriodOnDayState = (person: CalendarEmployee, day: number): EmployeeState|undefined => 
{
  const dateKey = formatDateKey(day)
  return person.statuses.find(period => isDateInPeriod(dateKey, period))
}

const getPeriodForDay = (person: CalendarEmployee, day: number): EmployeeState | null => 
{
  const dateKey = formatDateKey(day)
  return person.statuses.find(period => isDateInPeriod(dateKey, period)) || null
}
const handleCellHover = (person: CalendarEmployee, day: number): void => 
{
  hoveredColumn.value = day
}

const handleCellLeave = (): void => 
{
  hoveredColumn.value = null
}

const getDayCellClass = (person: CalendarEmployee, day: number): string => 
{
  const classes = ['day-cell']
  if (isWeekend(day)) classes.push('weekend')
  if (hasPeriodOnDay(person, day)) classes.push('has-period')
  if (isCurrentDay(day)) classes.push('current-day')
  if (currentPerson.value?.employee.id === person.employee.id) classes.push('current-person-cell')
  if (hoveredColumn.value === day) classes.push('column-hovered')
  return classes.join(' ')
}

const getDayHeaderClass = (day: number): string => {
  const classes = ['day-header']
  if (isWeekend(day)) classes.push('weekend')
  if (isCurrentDay(day)) classes.push('current-day')
  if (hoveredColumn.value === day) classes.push('column-hovered')
  return classes.join(' ')
}

const getDayCellColor = (person: CalendarEmployee, day: number): string => 
{
  const state = hasPeriodOnDayState(person, day)
  if (state) 
  {
    const status = get_status(state.status_id);
    if(status)
      return ColorHelper.setOpacity(status.color, 0.5);
  }
  return ''
}
const getDayCellLogo = (person: CalendarEmployee, day: number): string|null|undefined => 
{
  const state = hasPeriodOnDayState(person, day)
  if (state) 
  {
    const status = get_status(state.status_id);
    if(status)
      return status.logo;
  }
}
const getTooltipContent = (person: CalendarEmployee, day: number): string => {
  const period = getPeriodForDay(person, day)
  const date = new Date(
    currentDate.value.getFullYear(),
    currentDate.value.getMonth(),
    day
  )
  if (period) {
    return `${person.employee.surname} ${person.employee.first_name[0]}.${person.employee.second_name[0]}. (${get_status(period.status_id)?.status}): ${period.start_date?.to_string(DateFormat.DotDate)} - ${period.end_date?.to_string(DateFormat.DotDate)}`
  }
  
  return `${person.employee.surname}: ${date.toLocaleDateString('ru-RU')}`
}

const isCurrentDay = (day: number): boolean => 
{
  const today = new Date()
  return (
    today.getDate() === day &&
    today.getMonth() === currentDate.value.getMonth() &&
    today.getFullYear() === currentDate.value.getFullYear()
  )
}

const handleCellClick = (person: CalendarEmployee, day: number): void => 
{
  const date = new Date(
    currentDate.value.getFullYear(),
    currentDate.value.getMonth(),
    day
  )
  const period = getPeriodForDay(person, day)
  
  //selectedCell.value = { person, day, period }
  //showCellModal.value = true
  currentPerson.value = person;
  console.log("Выбрана ячейка")
  emit('cellClick', person, date, period)
}


const selectPerson = (person: CalendarEmployee): void => 
{
  currentPerson.value = person
  emit('personSelect', person)
}

const formatModalDate = (day: number): string => 
{
  const date = new Date(
    currentDate.value.getFullYear(),
    currentDate.value.getMonth(),
    day
  )
  return date.toLocaleDateString('ru-RU', 
  {
    weekday: 'long',
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  })
}


</script>

 <style scoped lang="scss">
 $header-height: 40px;
 $row-height: 36px; 
 $weekend-color: rgba(54, 52, 52, 0.507); 
 $today-color: rgba(119, 245, 125, 0.307);
 $selected-row-color: rgba(120, 247, 209, 0.1);

.calendar-matrix 
{
  max-width: 100%;
  margin: 0 auto;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;

  .calendar-header 
  {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 24px;
    padding: 16px;
    background: var(--n-color);
    border-radius: 8px;
    border: 1px solid var(--n-border-color);

    h2 
    {
      margin: 0;
      color: var(--n-text-color);
      text-transform: capitalize;
      font-size: 1.5em;
      font-weight: 600;
    }
  }
}

.matrix-container 
{
  display: flex;
  background: var(--n-color);
  border: 1px solid var(--n-border-color);
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 2px 8px var(--n-box-shadow);
}

.column-hovered 
{
  background-color: rgba(0, 123, 255, 0.1) !important;
  position: relative;
}

.people-column 
{
  min-width: 200px;
  border-right: 2px solid var(--n-border-color);
  background: var(--n-color-secondary);

  .person-header 
  {
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    font-weight: 600;
    text-align: center;
    border-bottom: 2px solid var(--n-border-color);
    font-size: 14px;
    height: $header-height;
  }

  .person-row 
  {
    display: flex;
    align-items: center;
    padding: 8px 16px;
    border-bottom: 1px solid var(--n-border-color);
    transition: all 0.3s ease;
    cursor: pointer;
    //height: calc($header-height - 28px);
    height: $row-height; // Используем единую высоту
    box-sizing: border-box;

    &.current-person 
    {
      background-color: $selected-row-color;
      border-left: 3px solid var(--n-color-primary);
    }

    &:hover 
    {
      background: rgba(65, 63, 63, 0.432);
    }

    &:last-child 
    {
      border-bottom: none;
    }

    .person-info 
    {
      display: flex;
      align-items: center;
      gap: 12px;
      width: 100%;
    }

    .person-color 
    {
      width: 16px;
      height: 16px;
      border-radius: 4px;
      border: 1px solid var(--n-border-color);
      flex-shrink: 0;
    }

    .person-name 
    {
      font-weight: 500;
      color: var(--n-text-color);
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      font-size: 14px;
    }
  }
}

.dates-header .column-hovered .day-header-cell 
{
  background: rgba(0, 123, 255, 0.082) !important;
  color: var(--n-color-primary) !important;
}

// Для ячеек с периодами - делаем цвет более насыщенным при hover, пока не делаем, оставим если будет нужно
//.day-cell.column-hovered.has-period 
//{
  //z-index: 3;
  //box-shadow: 0 0 0 2px var(--n-color-primary);
//}

.dates-container 
{
  flex: 1;
  overflow-x: auto;
  overflow-y: hidden;
  min-width: 0;

  .dates-header 
  {
    border-bottom: 2px solid var(--n-border-color);
    background: var(--n-color-secondary);

    .day-header-cell 
    {
      text-align: center;
      font-size: 10px;
      font-weight: 600;
      color: var(--n-text-color);
      border-right: 1px solid var(--n-border-color);
      height:  $header-height;
      align-items: center; // Центрируем по вертикали
      justify-content: center;
      box-sizing: border-box; // Важно!
      display: flex;
      flex-direction: column;

      .day-number 
      {
        font-size: 14px;
        font-weight: 700;
        margin-bottom: 2px;
        line-height: 1;
      }

      .day-name 
      {
        font-size: 12px;
        opacity: 0.8;
        line-height: 1;
      }
    }

    .weekend 
    {
       background: $weekend-color;
    }

    .current-day 
    {
      background: $today-color;
    }
  }

  .dates-grid {
    .person-row {
      border-bottom: 1px solid rgb(126, 122, 122);
      height: $row-height; // Та же высота что и у person-row
      box-sizing: border-box;

      &:last-child 
      {
        border-bottom: none;
      }

      .day-cell 
      {
        border-right: 1px solid var(--n-border-color);
        border-bottom: 1px solid rgb(126, 122, 122);
        cursor: pointer;
        transition: all 0.2s ease;
        position: relative;
        height:  $row-height;
        display: flex;
        width: 100%;
        align-items: center;
        box-sizing: border-box;
        justify-content: center;

        &.weekend 
        {
          background: $weekend-color;
          //border-bottom: inherit;
          //  &:last-child {
          //   border-bottom: 1px solid rgb(126, 122, 122);
          //  }
        }

        // &.has-period 
        // {
        //   //border: 1px solid rgba(122, 243, 227, 0.568);
        // }

        &.current-day 
        {
          background: $today-color;
        }

        &.current-person-cell 
        {
          background-color: $selected-row-color;
        }
        .period-marker 
        {
          width: 80%; 
          height: 80%; 
          display: flex;
          align-items: center;
        }
      }
    }
  }
}

.cell-modal-content {
  .cell-periods {
    margin-top: 12px;
  }
}

// Гарантия одинакового размера ячеек
.n-grid {
  display: grid;
  grid-template-columns: repeat(v-bind(daysInMonth), minmax(20px, 20px));
  
  .n-gi {
    min-width: 0;
    width: 100%;
    height: 100%;
  }
}

// Фиксированная высота для всех строк сетки
.dates-header .n-grid,
.dates-grid .n-grid {
  grid-template-columns: repeat(v-bind(daysInMonth), minmax(v-bind(cellSize + 'px'), 1fr));
  height: $header-height; // Для заголовков
}

.dates-grid .n-grid {
  height: $row-height; // Для строк с данными
}

.filter-panel
{
  display: flex;
  flex-direction: row;
  gap: 10px;
  margin-bottom: 5px;
}
</style>