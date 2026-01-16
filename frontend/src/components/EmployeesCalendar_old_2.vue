<template lang="pug">
.calendar-matrix
  .calendar-header
    button.prev-month(@click="prevMonth") ←
    h2 {{ currentMonthYear }}
    button.next-month(@click="nextMonth") →

  .matrix-container
    .people-column
      .person-header Люди
      .person-row(
        v-for="person in people"
        :key="person.id"
        :class="{ 'current-person': currentPerson?.id === person.id }"
      )
        .person-info
          .person-color(:style="{ backgroundColor: person.color }")
          .person-name {{ person.name }}

    .dates-container
      .dates-header
        .day-cell(
          v-for="day in daysInMonth"
          :key="day"
          :class="getDayHeaderClass(day)"
        ) 
          | {{ day }}
          br
          | {{ getDayName(day) }}

      .dates-grid
        .person-row(
          v-for="person in people"
          :key="person.id"
        )
          .day-cell(
            v-for="day in daysInMonth"
            :key="day"
            :class="getDayCellClass(person, day)"
            :style="{ backgroundColor: getDayCellColor(person, day) }"
            @click="handleCellClick(person, day)"
            @mouseenter="handleCellHover(person, day)"
            @mouseleave="handleCellLeave"
          )
            .cell-content
              .period-marker(v-if="hasPeriodOnDay(person, day)")

  .calendar-tooltip(
    v-if="tooltip.visible"
    :style="tooltip.style"
  )
    .tooltip-person {{ tooltip.personName }}
    .tooltip-date {{ tooltip.date }}
    .tooltip-period(v-if="tooltip.period") 
      | {{ tooltip.period.startDate }} - {{ tooltip.period.endDate }}

  .calendar-legend(v-if="people.length")
    h3 Легенда
    .legend-items
      .legend-item(
        v-for="person in people"
        :key="person.id"
        @click="selectPerson(person)"
        :class="{ 'current-person': currentPerson?.id === person.id }"
      )
        .legend-color(:style="{ backgroundColor: person.color }")
        .legend-name {{ person.name }}
</template>
<script setup lang="ts">
import { ref, computed, onMounted, reactive } from 'vue'

// Типы
interface Person {
  id: string
  name: string
  color: string
  periods: DatePeriod[]
}

interface DatePeriod {
  startDate: string // YYYY-MM-DD
  endDate: string // YYYY-MM-DD
  personId: string
  type?: string
}

interface Tooltip {
  visible: boolean
  personName: string
  date: string
  period: DatePeriod | null
  style: {
    left: string
    top: string
  }
}

// Props
interface Props {
  people: Person[]
}

const props = defineProps<Props>()

// Emits
const emit = defineEmits<{
  cellClick: [person: Person, date: Date, period: DatePeriod | null]
  personSelect: [person: Person]
}>()

// Data
const currentDate = ref(new Date())
const currentPerson = ref<Person | null>(null)
const tooltip = reactive<Tooltip>({
  visible: false,
  personName: '',
  date: '',
  period: null,
  style: { left: '0px', top: '0px' }
})

// Computed
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

const monthStart = computed(() => {
  const year = currentDate.value.getFullYear()
  const month = currentDate.value.getMonth()
  return new Date(year, month, 1)
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

const getDayHeaderClass = (day: number): string => {
  const classes = ['day-header']
  if (isWeekend(day)) classes.push('weekend')
  if (isCurrentDay(day)) classes.push('current-day')
  return classes.join(' ')
}

const formatDateKey = (day: number): string => {
  const year = currentDate.value.getFullYear()
  const month = currentDate.value.getMonth() + 1
  return `${year}-${month.toString().padStart(2, '0')}-${day.toString().padStart(2, '0')}`
}

const isDateInPeriod = (date: string, period: DatePeriod): boolean => {
  return date >= period.startDate && date <= period.endDate
}

const hasPeriodOnDay = (person: Person, day: number): boolean => {
  const dateKey = formatDateKey(day)
  return person.periods.some(period => isDateInPeriod(dateKey, period))
}

const getPeriodForDay = (person: Person, day: number): DatePeriod | null => {
  const dateKey = formatDateKey(day)
  return person.periods.find(period => isDateInPeriod(dateKey, period)) || null
}

const getDayCellClass = (person: Person, day: number): string => {
  const classes = ['day-cell']
  if (isWeekend(day)) classes.push('weekend')
  if (hasPeriodOnDay(person, day)) classes.push('has-period')
  if (isCurrentDay(day)) classes.push('current-day')
  return classes.join(' ')
}

const getDayCellColor = (person: Person, day: number): string => {
  if (hasPeriodOnDay(person, day)) {
    return person.color
  }
  return ''
}

const isCurrentDay = (day: number): boolean => {
  const today = new Date()
  return (
    today.getDate() === day &&
    today.getMonth() === currentDate.value.getMonth() &&
    today.getFullYear() === currentDate.value.getFullYear()
  )
}

const handleCellClick = (person: Person, day: number): void => {
  const date = new Date(
    currentDate.value.getFullYear(),
    currentDate.value.getMonth(),
    day
  )
  const period = getPeriodForDay(person, day)
  emit('cellClick', person, date, period)
}

const handleCellHover = (person: Person, day: number, event: MouseEvent): void => {
  const date = new Date(
    currentDate.value.getFullYear(),
    currentDate.value.getMonth(),
    day
  )
  const period = getPeriodForDay(person, day)
  
  tooltip.personName = person.name
  tooltip.date = date.toLocaleDateString('ru-RU')
  tooltip.period = period
  tooltip.style.left = `${event.clientX + 10}px`
  tooltip.style.top = `${event.clientY + 10}px`
  tooltip.visible = true
}

const handleCellLeave = (): void => {
  tooltip.visible = false
}

const selectPerson = (person: Person): void => {
  currentPerson.value = person
  emit('personSelect', person)
}

// Lifecycle
onMounted(() => {
  if (props.people.length > 0) {
    currentPerson.value = props.people[0]
  }
})
</script>

 <style scoped lang="scss">

 $date-min-width: 40px;
 $date-height: 40px;
.calendar-matrix {
  max-width: 100%;
  margin: 0 auto;
  padding: 20px;
  font-family: Arial, sans-serif;

  .calendar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 20px;
    padding: 15px;
    background: #f8f9fa;
    border-radius: 8px;
    border: 1px solid #e9ecef;

    h2 {
      margin: 0;
      color: #495057;
      text-transform: capitalize;
      font-size: 1.5em;
    }

    button {
      background: #007bff;
      color: white;
      border: none;
      padding: 10px 20px;
      border-radius: 6px;
      cursor: pointer;
      font-size: 16px;
      transition: all 0.3s ease;

      &:hover {
        background: #0056b3;
        transform: translateY(-1px);
      }
    }
  }
}

.matrix-container {
  display: flex;
  background: white;
  border: 1px solid #e9ecef;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.people-column {
  min-width: 200px;
  border-right: 2px solid #e9ecef;
  background: #f8f9fa;

  .person-header {
    padding: 15px;
    background: #495057;
    color: white;
    font-weight: bold;
    text-align: center;
    border-bottom: 1px solid #dee2e6;
  }

  .person-row {
    display: flex;
    align-items: center;
    padding: 12px 15px;
    border-bottom: 1px solid #e9ecef;
    transition: background-color 0.3s ease;

    &.current-person {
      background: #e3f2fd;
      border-left: 3px solid #007bff;
    }

    &:hover {
      background: #e9ecef;
    }

    .person-info {
      display: flex;
      align-items: center;
      gap: 10px;
      width: 100%;
    }

    .person-color {
      width: 16px;
      height: 16px;
      border-radius: 3px;
      border: 1px solid #dee2e6;
      flex-shrink: 0;
    }

    .person-name {
      font-weight: 500;
      color: #495057;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }
  }
}

.dates-container {
  flex: 1;
  overflow-x: auto;

  .dates-header {
    display: flex;
    background: #f8f9fa;
    border-bottom: 2px solid #e9ecef;

    .day-header {
      flex: 1;
      min-width: $date-min-width;
      padding: 10px 5px;
      text-align: center;
      font-size: 12px;
      font-weight: bold;
      color: #495057;
      border-right: 1px solid #e9ecef;

      &.weekend {
        background: #f8d7da;
        color: #721c24;
      }

      &.current-day {
        background: #007bff;
        color: white;
      }

      &:last-child {
        border-right: none;
      }
    }
  }
  .dates-grid {
    .person-row {
      display: flex;
      border-bottom: 1px solid #e9ecef;

      &:last-child {
        border-bottom: none;
      }

      .day-cell {
        flex: 1;
        min-width: $date-min-width;
        height:  $date-height;
        border-right: 1px solid #e9ecef;
        cursor: pointer;
        transition: all 0.3s ease;
        position: relative;

        &.weekend {
          background: #f8f9fa;
        }

        &.has-period {
          border: 2px solid #495057;
        }

        &.current-day {
          border: 2px solid #007bff;
        }

        &:hover {
          transform: scale(1.05);
          z-index: 1;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
        }

        &:last-child {
          border-right: none;
        }

        .cell-content {
          width: 100%;
          height: 100%;
          display: flex;
          align-items: center;
          justify-content: center;
        }

        .period-marker {
          width: 24px;
          height: 24px;
          border-radius: 50%;
          background: rgba(255, 255, 255, 0.8);
          border: 2px solid currentColor;
        }
      }
    }
  }
}

.calendar-tooltip {
  position: fixed;
  background: rgba(0, 0, 0, 0.9);
  color: white;
  padding: 12px;
  border-radius: 6px;
  font-size: 14px;
  z-index: 1000;
  pointer-events: none;
  max-width: 250px;

  .tooltip-person {
    font-weight: bold;
    margin-bottom: 4px;
  }

  .tooltip-date {
    margin-bottom: 4px;
  }

  .tooltip-period {
    font-size: 12px;
    opacity: 0.8;
  }
}

.calendar-legend {
  margin-top: 20px;
  padding: 20px;
  background: #f8f9fa;
  border-radius: 8px;
  border: 1px solid #e9ecef;

  h3 {
    margin: 0 0 15px 0;
    color: #495057;
  }

  .legend-items {
    display: flex;
    flex-wrap: wrap;
    gap: 15px;
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    background: white;
    border-radius: 6px;
    border: 1px solid #dee2e6;
    cursor: pointer;
    transition: all 0.3s ease;

    &.current-person {
      background: #e3f2fd;
      border-color: #007bff;
    }

    &:hover {
      background: #e9ecef;
      transform: translateY(-1px);
    }
  }

  .legend-color {
    width: 20px;
    height: 20px;
    border-radius: 4px;
    border: 1px solid #dee2e6;
  }

  .legend-name {
    font-size: 14px;
    color: #495057;
    font-weight: 500;
  }
}

// Адаптивность
@media (max-width: 768px) {
  .calendar-matrix {
    padding: 10px;

    .calendar-header {
      padding: 10px;
      
      h2 {
        font-size: 1.2em;
      }
      
      button {
        padding: 8px 16px;
        font-size: 14px;
      }
    }
  }

  .people-column {
    min-width: 150px;

    .person-row {
      padding: 8px 10px;

      .person-name {
        font-size: 14px;
      }
    }
  }

  .dates-container {
    .dates-header .day-header {
      min-width: 50px;
      font-size: 11px;
      padding: 8px 3px;
    }

    .dates-grid .person-row .day-cell {
      min-width: 50px;
      height: 50px;
    }
  }
}
</style>