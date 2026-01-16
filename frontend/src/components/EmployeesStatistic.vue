<template lang="pug">
n-card.overall-statistics(size="small" v-if="props.employees")
  n-grid(:cols="24" :x-gap="16" :y-gap="12")
    //- Заголовок
    n-gi(:span="24")
      .header
        n-text(strong depth="1" style="font-size: 18px") 📊 Статистика сотрудников
        n-date-picker(
            v-model:value="selected_date"
            clearable
            format="dd.MM.yyyy"
            type="date"
          )
        n-button(@click="show_modal = true") Настроить выборку
    n-gi(:span="24")
      n-table(:bordered="false" :single-line="false")
        thead
          tr
            th.dep-name
            th Всего
            th На месте
            th(v-if="statistic" v-for="value in get_categories") {{ value }}
        tbody(v-if="statistic")

          tr(v-for="department in statistic.departments" :key="department.department_id")
            td.dep-name {{ department.department_name }}
            td {{ department.employees_count }}
            td
              progress-bar(:current_count="department.employees_on_work" :total_count="department.employees_count")
            td(v-for="cat in department.groups" :key="cat.name")
              progress-bar(v-if="cat.status_counts > 0" 
                :current_count="cat.status_counts" 
                :total_count="department.employees_count" 
                :employees="cat.employees_list")
          tr
            td.dep-name Итого
            td {{ statistic.total_employees_count}}
            td 
              progress-bar(:current_count="statistic.total_on_work_count" :total_count="statistic.total_employees_count")
            td(v-for="cat in get_categories")
              progress-bar(v-if="statistic.total_count_by_category[cat] > 0"
               :current_count="statistic.total_count_by_category[cat]" 
               :total_count="statistic.total_employees_count"
               :employees="get_employees_fio_by_category(cat)")

//- Модальное окно добавления/редактирования
n-modal(
  v-model:show="show_modal"
  title="Редактирование выборки"
  preset="dialog"
  :style="{ width: '600px' }"
  positive-text="Сохранить"
  negative-text="Отмена"
  @positive-click="handle_save"
  @negative-click="handle_cancel"
)

  n-grid(:cols="3" :x-gap="24")
    n-gi(span="3")
      n-dynamic-input(v-model:value="statistic_editor_values" :on-create="on_create" show-sort-button)
        template(#default="{ value }")
          .selectors-editor
            n-input(v-model:value="value.name" type="text")
            .checkboxes
              n-checkbox(v-for="(s, i) in statusOptions" :key="s.value" :checked="value.status.has(s.value)"  @update:checked="(checked) => handle_checked_change(value, s.value, checked)") {{ s.label }}


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
  NCheckbox,
  NDivider,
  NTable,
  NInput,
  NModal,
  NButton,
  NDynamicInput,
  NDatePicker
} from 'naive-ui'
import { DateFormat, DateTime } from '@/services/date';
import { type Employees } from '@/types/employees';
import { useDictionaries } from '@/composables/useDictionaries';
import { load_from_localstorage, save_to_localstorage } from '@/services/helpers';
import { type Fio, type DepartmentsStatistic } from '@/types/statistic';
import { http_sevice } from '@/services/http_service/http_service';
import ProgressBar from './ProgressBar.vue';
import EmployeesList from './EmployeesList.vue';

interface StatisticItem
{
  label: string
  value: number
  percentage: string
  color: string
  children?: StatisticItem[]
}
interface StatisticField
{
  name: string,
  status: Set<string>
}

</script>
<script lang="ts" setup>
const props = defineProps<{
  employees?: Employees
}>()
const { statuses, statusOptions, get_status, get_department, departmentOptions, clinics, clinicOptions } = useDictionaries();
const statistic = ref<DepartmentsStatistic>();
const show_modal = ref(false);
const statistic_values = ref<StatisticField[]>([]);
const statistic_editor_values = ref<StatisticField[]>([]);
const selected_date = ref<number|null>(null);
const load_statistic = async (request: {name: string;statuses: string[] }[]) => 
{
  if(request.length == 0)
    console.warn("Не определены поля запроса для статистики!");
  const result = await http_sevice.statistic_service.employees_current_states(request, selected_date.value ?? undefined);
  statistic.value = result;
}
const get_employees_fio_by_category = (cat: string) =>
{
  let fios: Fio[] = [];
  if(statistic.value)
  {
    for(const d of statistic.value.departments)
    {
      for(const g of d.groups)
      {
        if(g.name == cat)
        {
          fios = fios.concat(...g.employees_list)
        }
      }
    }
    return fios;
  }
}

const on_create = () => 
{
  return {
    name: "Болезнь",
    status: new Set([])
  }
}

const get_categories = computed(() =>
{
  if(statistic.value)
  {
    const keys = [...new Set(statistic.value.departments.map(s=>s.groups.map(g=>g.name)).flat())]
    return keys;
  }
  else
  {
    return []
  }
})

const handle_checked_change = (value: StatisticField, status: string, checked: boolean) => 
{
  if (checked) 
  {
    value.status.add(status);
  } 
  else 
  {
    value.status.delete(status);
  }
}



onMounted(async ()=>
{
  const values = load_from_localstorage<{name: string, status: string[]}[]>('statistic');
  if(values)
  {
    statistic_values.value = values.map(m=>
    {
      return {
        name: m.name,
        status: new Set(m.status)
      }
    });
    statistic_editor_values.value = statistic_values.value;
  }
})

watch(statistic_values, async (n) =>
{
    const request = n.map(m=>
    {
      return {
        name: m.name,
        statuses: Array.from(m.status)
      }
    });
    await load_statistic(request);
})

watch(selected_date, async (n) =>
{
    const request = statistic_values.value.map(m=>
    {
      return {
        name: m.name,
        statuses: Array.from(m.status)
      }
    });
    await load_statistic(request);
})




const handle_save = () =>
{
  console.log(statistic_editor_values.value);
  save_to_localstorage('statistic', statistic_editor_values.value.map(v=>
    {
      return {
        name: v.name,
        status: Array.from(v.status)
      }
    }
  ));
  statistic_values.value = statistic_editor_values.value;
  return true;
}
</script>
  
<style lang="scss" scoped>
.overall-statistics {
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
.selectors-editor
{
  display: flex;
  flex-direction: column;
  width: 100%;
}
.checkboxes
{
  display: flex;
  flex-direction: row;
  background-color: #35353598;
  flex-wrap: wrap;
}
// .pg
// {
//     position: relative;
//     bottom: 27.5px;
//     opacity: 0.15;
// }
// .percentage-and-value
// {
//   width: 100%;
//   display: flex;
//   align-items: center;
// }

/* Основные стили для таблицы */
:deep(.n-table) {
  width: 100%;
}

:deep(td) {

  vertical-align: middle !important;
  text-align: center;
  font-size: 20px;
  font-weight: 600;
}
:deep(th) {

  vertical-align: middle !important;
  text-align: center;
  font-size: 18px;
}

:deep(td div) {

  display: block !important;
}


/* Стили для ячеек с прогрессбаром */
:deep(.progress-cell) {
  position: relative;
  padding: 0 !important; /* Убираем внутренние отступы */
  margin: 0 !important;
}

:deep(.progress-wrapper) {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px 8px; /* Отступы внутри ячейки */
}

:deep(.progress-container) {
  position: relative;
  width: 100%;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0 auto;
}

/* Стили для прогрессбара (фон) */
:deep(.pg) {
  position: absolute !important;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  width: 100% !important;
  height: 100% !important;
  margin: 0 !important;
  z-index: 1; /* Прогрессбар позади текста */
}

:deep(.pg .n-progress-graph) {
  width: 100% !important;
  height: 100% !important;
}

:deep(.pg .n-progress-graph-line) {
  border-radius: 4px !important;
  height: 100% !important;
}

/* Стили для текста поверх прогрессбара */
:deep(.value-overlay) {

  z-index: 2; /* Текст поверх прогрессбара */
  font-weight: 600;
  font-size: 18px;
  color: #111111;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
  pointer-events: none;
  padding: 0 8px;
}

// /* Цвета прогрессбаров */
// :deep(.pg .n-progress-graph-line-fill) {
//   background: linear-gradient(90deg, #1890ff, #40a9ff) !important;
// }

// :deep(.pg.status-success .n-progress-graph-line-fill) {
//   background: linear-gradient(90deg, #52c41a, #73d13d) !important;
// }

// :deep(.pg.status-warning .n-progress-graph-line-fill) {
//   background: linear-gradient(90deg, #faad14, #ffc53d) !important;
// }

// :deep(.pg.status-error .n-progress-graph-line-fill) {
//   background: linear-gradient(90deg, #ff4d4f, #ff7875) !important;
// }

.dep-name
{
  background-color: var(--n-th-color);
}

/* Адаптивность */
@media (max-width: 768px) {
  :deep(.progress-container) {
    height: 24px;
  }
  
  :deep(.value-overlay) {
    font-size: 11px;
    padding: 0 4px;
  }
  
  :deep(td) {
    padding: 6px 2px !important;
    height: 40px !important;
  }
}


grid-layout {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1px;
  background-color: #e1e5e900;
  border: 1px solid #e1e5e9;
  border-radius: 8px;
  overflow: hidden;
}

.grid-layout > * {
  background-color: rgba(255, 255, 255, 0);
  padding: 12px 16px;
  display: flex;
  align-items: center;
}

// Заголовки статусов
.status-header {
  grid-column: 1 / -1;
  background-color: #36363600 !important;
  border-bottom: 2px solid #e9ecef;
  font-weight: bold;
  font-size: 1.1em;
}

// Подзаголовки
.department-header,
.count-header,
.employees-header {
  background-color: #f1f3f400 !important;
  font-weight: 600;
  border-bottom: 1px solid #e9ecef;
}

// Итоговые строки
.total-row {
  font-weight: bold;
  color: #1890ff;
}

.employees-count {
  cursor: help;
  color: #1890ff;
  text-decoration: underline dotted;
}

// Адаптивность
@media (max-width: 768px) {
  .grid-layout {
    grid-template-columns: 1fr;
  }
  
  .status-header {
    grid-column: 1;
  }
}
</style>