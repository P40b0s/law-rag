<template lang="pug">
div 
    .filter-container
        .year-select
            n-text(strong) Год:
            n-select(
                v-model:value="selected_year"
                :options="year_options"
                style="width: 120px; margin-left: 8px;"
                placeholder="Выберите год"
            )
        
        .month-select
            n-checkbox-group.month-group(v-model:value="selected_months" style="margin-left: 8px;")
                n-space(horizontal)
                n-checkbox(
                    v-for="month in month_options"
                    :key="month.value"
                    :value="month.value"
                    :label="month.label")
            n-button.accept-date-btn(@click="apply_date_filters" type="primary") Применить
            

        .status-select
            n-checkbox-group.month-group(v-model:value="selected_statuses" style="margin-left: 8px;")
                n-space(horizontal)
                n-checkbox(
                    v-for="status in statusOptions"
                    :key="status.value"
                    :value="status.value"
                    :label="status.label"
            )
        .filter-panel
            n-select(v-model:value="selected_department"
                    clearable
                    :options="departmentOptions"
                    placeholder="Фильтр по отделу")

    #chart
        apexchart(type="bar" height="450" :options="options" :series="get_data")
</template>
    
<script lang="ts">
import { ref, type Component, watch, inject, onMounted, onUnmounted, computed, onBeforeUnmount, h, toRefs, reactive } from 'vue';
import { type Events, type Emitter } from '../services/emitter';
import { NForm, NTabs, NTabPane, NFormItem, NInput, NButton, NCheckbox, NCheckboxGroup, NText, NSpace, NSelect, NIcon, NCard, darkTheme,  type SelectOption } from 'naive-ui';
import { notify_service } from '@/services/notification_service';
import ApexCharts, { type ApexOptions} from 'apexcharts'

import { http_sevice } from '@/services/http_service/http_service';
import { useDictionaries } from '@/composables/useDictionaries';
import { type CalendarEmployee } from '@/types/employees';
import { DateTime } from '@/services/date';
import { useTheme } from '@/composables/useTheme';
import { save_to_localstorage, sleepNow, load_from_localstorage } from '@/services/helpers';
interface ChartEmployeer
{
    name: string,
    color?: string,
    data: ChartData[]

}
interface ChartData
{
    x: string,
    y: number,
}

</script>
<script lang="ts" setup>
const emitter = inject<Emitter<Events>>('emitter') as Emitter<Events>;
const { statuses, loading, statusOptions, get_status, departmentOptions, clinics, clinicOptions } = useDictionaries();
const {get_current_theme} = useTheme()

const selected_department = ref<string | null>(null)
const employees = ref<CalendarEmployee[]>([])
const selected_year = ref<number>(new Date().getFullYear())
const selected_months = ref<number[]>([])
const selected_statuses = ref<string[]>([]);

const load_employees = async () =>
{
    employees.value = [];
    const year = selected_year.value;
    for(const m of selected_months.value)
    {
        const peoples = await http_sevice.employees_service.get_calendar_employees(year, m, selected_department.value ?? undefined);
        if(peoples)
        {
            if(employees.value.length == 0)
                employees.value = peoples;
            else
            for(const e of peoples)
            {
                const index = employees.value.findIndex(f=>f.employee.id == e.employee.id);
                if(index != -1)
                {
                    employees.value[index].statuses = employees.value[index].statuses.concat(e.statuses);
                }
            }
        }
    }
}

watch(selected_department, async () =>
{
    await load_employees();
})
watch(selected_statuses, async () =>
{
    save_to_localstorage('statistic-status-options', selected_statuses.value);
})
watch(statusOptions, () =>
{
    const s: string[]|undefined = load_from_localstorage('statistic-status-options');
    if(s)
        selected_statuses.value = s;
    else
        selected_statuses.value = statusOptions.value.map(v=>v.value);
}, {immediate: true})


const get_data = computed<ChartEmployeer[]>(()  =>
{
    return selected_statuses.value.map(s=>
    {
        return {
            name: get_status(s)?.status,
            data: get_employeers_data(s),
            color: get_status(s)?.color
        } as ChartEmployeer
    })
})

const get_employeers_data = (status_id: string): ChartData[] =>
{
    return employees.value.map(e=>
        {
            let statuses = e.statuses.filter(f=>f.status_id == status_id);
            const summ = statuses.reduce((acc: number, current) => {
                if(current.start_date && current.end_date)
                {
                    const days = current.start_date?.days_between(current.end_date as DateTime);
                    return acc+days;
                }
                else return acc;
               
            }, 0);
            return {
                x: get_fio(e.employee),
                y: summ
            } as ChartData
        }
    )
}
const get_fio = (employee: {
    id: string;
    first_name: string;
    second_name: string;
    surname: string;
    department: string;
}) => `${employee.surname} ${employee.first_name[0]}.${employee.second_name[0]}.`
  const employeeTotals = computed(() => employees.value.map((employee, empIndex) => 
  {
    return get_data.value.reduce((sum, series) => 
      sum + (series.data[empIndex]?.y || 0), 0
    );
  }))
   const maxXValue = computed(() => Math.max(...employeeTotals.value))
const annotations = computed<ApexAnnotations>(() =>
{
  const points: PointAnnotations[] = [];
  employees.value.forEach((employee, empIndex) =>
  {
    const totalDays = employeeTotals.value[empIndex];
    let point: PointAnnotations = 
    {
      x: totalDays + (maxXValue.value * 0.01) as number | string,//X - справа от бара
      y: 1,//get_fio(employees.value[empIndex].employee), // Y - имя сотрудника (категория)
      marker: {
        size: 0 // Скрываем маркер
      },
      label: {
        borderColor: '#00E396',
        offsetY: 5,
        offsetX: 0,
        style: {
          color: '#000000',
          background: '#00E396',
          fontSize: '14px',
          fontWeight: 'bold',
          
        },
          //text: `Всего: ${totalDays} ${get_day_suffix(totalDays)}`
          text: totalDays.toString()
        }
    }
    points.push(point)
  })
  return { points } as ApexAnnotations
})


const options =  computed<ApexOptions>(() =>
        {
            return {
                theme: {
                    mode: get_current_theme().value, 
                    palette: 'palette1', 
                },
                chart: {
                type: 'bar',
                height: 730,
                stacked: true,
                },
                plotOptions: {
                bar: {
                    horizontal: true,
                    barHeight: '100%',
                    dataLabels: {
                        position: 'center',
                        //total не пашет на горизонтальных барах
                        // total: {
                        //     enabled: true,
                        //     offsetX: 2,
                        //     style: {
                        //         fontSize: '13px',
                        //         fontWeight: 900
                        //     },
                        //     formatter: function (val) 
                        //     {
                        //         return val
                        //     }
                        // }
                    }
                },
                },
                dataLabels: {
                    enabled: true,
                    textAnchor: 'start',
                    style: {
                        fontSize: '16px', // размер шрифта на барах
                        fontWeight: 900,
                        colors: ['#262626']
                    },
                    // formatter: function(val: number) 
                    // {
                    //     return val.toString()
                    // },
                    offsetX: 0 // отступ от края
                },
                annotations: annotations.value,
                stroke: {
                width: 1,
                colors: ['#262626',]
                },
               
                xaxis: {
                    labels: {
                        style: {
                            fontSize: '16px',
                            fontFamily: 'Helvetica, Arial, sans-serif',
                            fontWeight: 400,
                        },
                },
                title: {
                    text: "Количество дней",
                    style:
                    {
                        fontSize: '18px',
                    }
                },
                },
                yaxis: {
                labels: {
                        style: {
                            fontSize: '16px',
                            fontFamily: 'Helvetica, Arial, sans-serif',
                            fontWeight: 400,
                        },
                        maxWidth: 400
                    },
                },
                tooltip: {
                    y: {
                        formatter: function (val) {
                        return val + " " + get_day_suffix(val)
                        }
                    }
                },
                fill: {
                    opacity: 1
                },
                legend: {
                    position: 'top',
                    horizontalAlign: 'center',

                    fontSize: '16px',
                    markers: {
                        offsetY: -2,
                        offsetX: -2
                    }
                },
            }
        })

const get_day_suffix = (days: number): string =>
{
  const last_digit = days % 10;
  const last_two_digits = days % 100;
  
  // Исключения: 11, 12, 13, 14
  if (last_two_digits >= 11 && last_two_digits <= 14) {
    return 'дней';
  }
  
  if (last_digit === 1) {
    return 'день';
  }
  
  if (last_digit >= 2 && last_digit <= 4) {
    return 'дня';
  }
  
  return 'дней';
}




//const chart_options = computed(() => options);

onMounted(async () => 
{
    const current_month = new Date().getMonth() + 1
    selected_months.value = [current_month]
    await load_employees()
})



// Year options (текущий год и несколько предыдущих/следующих)
const year_options = computed<SelectOption[]>(() => 
{
  const current_year = new Date().getFullYear()
  const years = []
  
  for (let i = current_year - 2; i <= current_year + 2; i++) 
  {
    years.push({
      label: i.toString(),
      value: i
    })
  }
  
  return years
})

// Month options
const month_options = computed(() => [
  { label: 'Янв', value: 1 },
  { label: 'Фев', value: 2 },
  { label: 'Мар', value: 3 },
  { label: 'Апр', value: 4 },
  { label: 'Май', value: 5 },
  { label: 'Июн', value: 6 },
  { label: 'Июл', value: 7 },
  { label: 'Авг', value: 8 },
  { label: 'Сен', value: 9 },
  { label: 'Окт', value: 10 },
  { label: 'Ноя', value: 11 },
  { label: 'Дек', value: 12 }
])

// Выбранные месяцы в читаемом формате
const selected_months_labels = computed(() => 
{
  return selected_months.value.map(month => 
    month_options.value.find(m => m.value === month)?.label || ''
  )
})


const apply_date_filters = async () => 
{
  await load_employees();
}

const reset_date_filters = () => 
{
  selected_year.value = new Date().getFullYear()
  const current_month = new Date().getMonth() + 1
  selected_months.value = [current_month]
  selected_department.value = null;
}

// watch([selected_year, selected_months, selected_department], () => 
// {
//   // Автоматическое применение фильтров при изменении
//   // или убрать, если хотите применять только по кнопке
//   if (selected_months.value.length > 0) 
//   {
//     console.log("auto date filter")
//     //apply_date_filters() // раскомментируйте для автоматического применения
//   }
// }, { deep: true })
</script>
    
<style lang="scss" scoped>
 .filter-container {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  background: var(--n-color);
  border-radius: 8px;
  border: 1px solid var(--n-border-color);
  flex-wrap: nowrap;

  .year-select {
    display: flex;
    align-items: center;
  }

  .month-select {
    display: flex;
    flex-direction: column;
    
    :deep(.n-checkbox-group) {
      display: flex;
      flex-wrap: wrap;
      width: 300px;
      gap: 8px;
    }
  }
  .status-select {
    display: flex;
    align-items: center;
  
    
    :deep(.n-checkbox-group) {
      display: flex;
      flex-wrap: wrap;
      width: 300px;
      gap: 8px;
    }
}

  .accept-date-btn
  {
    margin-top: 10px;
  }
}

// Альтернативный компактный стиль для месяцев
.month-checkboxes {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  gap: 8px;
  
  @media (max-width: 768px) {
    grid-template-columns: repeat(4, 1fr);
  }
  
  @media (max-width: 480px) {
    grid-template-columns: repeat(3, 1fr);
  }
}
.filter-panel
{
    flex-basis: 300px;
}
</style>