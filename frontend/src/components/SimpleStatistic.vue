<template lang="pug">
n-card.stacked-bar-statistics(size="small")
  n-grid(:cols="24" :x-gap="16" :y-gap="12")
    //- Заголовок
    n-gi(:span="24")
      .header
        n-text(strong depth="1" style="font-size: 18px") 📊 Статистика сотрудников
        n-tag(size="medium" type="primary") Всего: {{ total_employees }} чел.
    n-gi(:span="24")
      .detailed-legend
        .legend-category(v-for="category in statistic_data" :key="category.label")
          .category-header
            .color-dot(:style="{ background: category.color }")
            .label-header
              n-text(strong style="font-size: 16px") {{ category.label }}
              n-tag(size="medium" :type="get_tag_type(category.label)") 
                | {{ category.value }} ({{ category.percentage }})
          
          .category-breakdown(v-if="category.children && category.children.length > 0")
            .breakdown-item(
              v-for="item in category.children" 
              :key="item.label"
              :class="{ 'has-children': item.children && item.children.length > 0 }"
            )
              .breakdown-main
                n-text(depth="2" style="font-size: 14px") 
                  span(:style="{ color: item.color }") ▸ 
                  | {{ item.label }}: 
                n-text(strong style="font-size: 14px") {{ item.value }} чел.
                n-text(depth="3" style="font-size: 12px") ({{ item.percentage }})
              
              //- Вложенные дети (второй уровень)
              .breakdown-children(v-if="item.children && item.children.length > 0")
                .child-item(v-for="child in item.children" :key="child.label")
                  n-text(depth="3" style="font-size: 14px") 
                    span(:style="{ color: child.color }") ▸▸ 
                    | {{ child.label }}: 
                  n-text(strong style="font-size: 14px") {{ child.value }} чел.
                  n-text(depth="3" style="font-size: 12px") ({{ child.percentage }})
</template>
  
<script lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import {
  NTag,
  NProgress,
  NGi,
  NCard,
  NText,
  NStatistic,
  NGrid,
  NDivider
} from 'naive-ui'
import { DateFormat, DateTime } from '@/services/date';
import { type Employees } from '@/types/employees';
import { useDictionaries } from '@/composables/useDictionaries';

interface StatisticItem
{
  label: string
  value: number
  percentage: string
  color: string
  children?: StatisticItem[]
}

</script>
<script lang="ts" setup>
const props = defineProps<{
  employees: Employees
}>()
const { statuses, statusOptions, get_status, departmentOptions, clinics, clinicOptions } = useDictionaries();

const total_employees = computed(() => props.employees.length);
const sick_employees = computed(() => props.employees.filter(f=>get_status(f.status_id ?? "")?.is_disease).length);
const tracing_employees = computed(() => props.employees.filter(f=>get_status(f.status_id ?? "")?.tracing).length);
const working_employees = computed(() => props.employees.filter(f=> f.status_id == undefined || get_status(f.status_id)?.on_work_place).length);
const sick_percentage = computed(() => 
  total_employees.value > 0 ? (sick_employees.value / total_employees.value) * 100 : 0
)
const tracing_percentage = computed(() => 
  total_employees.value > 0 ? (tracing_employees.value / total_employees.value) * 100 : 0
)

const statistic_data = computed<StatisticItem[]>(() => 
{
  if (total_employees.value === 0) return []
  const normal_sick = computed(() => sick_employees.value - tracing_employees.value);
  const arr = other_statistic_data.value.concat(
  [
    {
      label: 'На рабочем месте',
      value: working_employees.value,
      percentage: total_employees.value > 0 ? `${((working_employees.value / total_employees.value) * 100).toFixed(1)}%` : '0%',
      color: '#18A058',
      children: workplace_statistic_data.value
    },
    {
      label: 'Заболевания',
      value: sick_employees.value,
      percentage: total_employees.value > 0 ? `${((sick_employees.value/ total_employees.value) * 100).toFixed(1)}%` : '0%',
      color: '#D03050',
      children: [
        {
          label: 'На контроле',
          value: tracing_employees.value,
          percentage: total_employees.value > 0 ? `${((tracing_employees.value / total_employees.value) * 100).toFixed(1)}%` : '0%',
          color: '#e66119',
          children: sick_trace_detail_data.value
        },
        {
          label: 'Другие',
          value: normal_sick.value,
          percentage: total_employees.value > 0 ? `${((normal_sick.value / total_employees.value) * 100).toFixed(1)}%` : '0%',
          color: '#edfb56',
          children: sick_other_detail_data.value
        }
      ]
    }
  ].filter(category => category.value > 0))
  return arr
})


const other_statistic_data = computed<StatisticItem[]>(() => 
{
  const c: Map<string, {count: number, percentage: string}> = new Map();
  for(let i: number = 0; i < props.employees.length; i++)
  {
    if(props.employees[i].status_id)
    {
      let status = get_status(props.employees[i].status_id as string)
      if(status)
      {
        if(!status.is_disease && !status.on_work_place)
        {
          const exists = c.get(status.status)
          let current_count = 1;
          if(exists)
          {
            current_count = exists.count + 1;
          }
          c.set(status.status, {count: current_count, percentage: `${((current_count / total_employees.value) * 100).toFixed(1)}%`})
        }
      }
    }
  }
  return Array.from(c, ([key, value]) => ({
    label: key,
    value: value.count,
    percentage: value.percentage,
    color: '#2888dc'
   }));
})

const workplace_statistic_data = computed<StatisticItem[]>(() => 
{
  const c: Map<string, {count: number, percentage: string}> = new Map();
  for(let i: number = 0; i < props.employees.length; i++)
  {
    if(props.employees[i].status_id)
    {
      let status = get_status(props.employees[i].status_id as string)
      if(status)
      {
        if(status.on_work_place)
        {
          const exists = c.get(status.status)
          let current_count = 1;
          if(exists)
          {
            current_count = exists.count + 1;
          }
          c.set(status.status, {count: current_count, percentage: `${((current_count / total_employees.value) * 100).toFixed(1)}%`})
        }
      }
    }
  }
  return Array.from(c, ([key, value]) => ({
    label: key,
    value: value.count,
    percentage: value.percentage,
    color: '#c5f24a'
   }));
})



const sick_trace_detail_data = computed<StatisticItem[]>(() => 
{
  const c: Map<string, {count: number, percentage: string}> = new Map();
  for(let i: number = 0; i < props.employees.length; i++)
  {
    if(props.employees[i].status_id)
    {
      let status = get_status(props.employees[i].status_id as string)
      if(status)
      {
        if(status.is_disease && status.tracing)
        {
          const exists = c.get(status.status)
          let current_count = 1;
          if(exists)
          {
            current_count = exists.count + 1;
          }
          c.set(status.status, {count: current_count, percentage: `${((current_count / total_employees.value) * 100).toFixed(1)}%`})
        }
      }
    }
  }
  return Array.from(c, ([key, value]) => ({
    label: key,
    value: value.count,
    percentage: value.percentage,
    color: '#18A058',
    children: undefined
   }));
})

const sick_other_detail_data = computed<StatisticItem[]>(() => 
{
  const c: Map<string, {count: number, percentage: string}> = new Map();
  for(let i: number = 0; i < props.employees.length; i++)
  {
    if(props.employees[i].status_id)
    {
      let status = get_status(props.employees[i].status_id as string)
      if(status)
      {
        if(status.is_disease && !status.tracing)
        {
          const exists = c.get(status.status)
          let current_count = 1;
          if(exists)
          {
            current_count = exists.count + 1;
          }
          c.set(status.status, {count: current_count, percentage: `${((current_count / total_employees.value) * 100).toFixed(1)}%`})
        }
      }
    }
  }
  return Array.from(c, ([key, value]) => ({
    label: key,
    value: value.count,
    percentage: value.percentage,
    color: '#18A058',
    children: undefined
   }));
})

// Вспомогательная функция для типа тега
const get_tag_type = (label: string): 'default' | 'success' | 'error' | 'warning' | 'info' => {
  const types: { [key: string]: string } = {
    'На рабочем месте': 'success',
    'Заболевания': 'error',
    'Командировки': 'info',
    'Отпуска': 'warning',
    'Прочие': 'default'
  }
  return (types[label] || 'default') as any
}

</script>
  
<style scoped lang="scss">
.stacked-bar-statistics {
  max-width: inherit;
  
  :deep(.n-card__content) {
    padding: 16px !important;
  }
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  flex-wrap: wrap;
  gap: 8px;
}

.detailed-legend {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  background: #3b3c3d;
  border-radius: 8px;
}

.legend-category {
  padding: 8px;
  background: rgb(31, 29, 29);
  border-radius: 6px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.category-header {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: nowrap;
  width: 100%;
}

.color-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  flex-shrink: 0;
}

.category-breakdown {
  display: grid;
  //grid-template-columns: repeat(auto-fit, minmax(80px, 1fr));
  grid-template-columns: auto;
  gap: 5px;
  padding-left: 10px;
  border-left: 2px solid #2c2e30;
  margin-left: 6px;
  margin-top: 4px;
}

.breakdown-item {
  padding: 8px;
  background: #2a2b2c;
  border-radius: 4px;
  
  &.has-children {
    background: #252627;
  }
}

.breakdown-main {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 4px;
  flex-wrap: wrap;
}

.breakdown-children {
  padding-left: 10px;
  margin-top: 6px;
  border-left: 1px solid #3a3b3c;
}

.child-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 0;
  flex-wrap: wrap;
  
  &:not(:last-child) {
    border-bottom: 1px solid #3a3b3c;
  }
}

// Адаптивность
@media (max-width: 768px) {
  .category-breakdown {
    grid-template-columns: auto;
  }
  
  .header {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }
  
  .breakdown-main,
  .child-item {
    align-items: flex-start;
    gap: 2px;
  }
}

// Анимации
.legend-category {
  animation: slideInRight 0.4s ease;
}

@keyframes slideInRight {
  from {
    opacity: 0;
    transform: translateX(20px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

// Ховер эффекты
.legend-category {
  transition: transform 0.2s ease, box-shadow 0.2s ease;
  
  &:hover {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }
}
.label-header
{
  width: inherit;
  display: flex;
  flex-direction: row;
  justify-content:space-between;
  align-items: center;
}
</style>