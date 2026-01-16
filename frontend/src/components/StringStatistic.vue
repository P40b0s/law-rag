<template lang="pug">
n-card.string-statistic(size="small")
  .tags
    template(v-for="status in current_statuses" :key="status.status_name")
      tag-with-progress(
        :color="status.color"
        :logo="status.logo"
        :status_name="`${status.status_name} ${status.count} (${status.percentage_str}%)`"
        :percentage="status.percentage"
        :count="status.count"
      )
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
import TagWithProgress from './TagWithProgress.vue';

interface StatisticItem
{
  status_name: string
  color: string,
  logo: string|null|undefined,
  percentage_str: string,
  percentage: number,
  count: number
}

</script>
<script lang="ts" setup>
const props = defineProps<{
  employees: Employees
}>()
const { statuses, statusOptions, get_status, departmentOptions, clinics, clinicOptions } = useDictionaries();

const total_employees = computed(() => props.employees.length);
const current_statuses = computed(() => 
{
  const map: Map<string, StatisticItem> = new Map();
  props.employees.forEach(emp => 
  {
    if(emp.status_id)
    {
      const map_item = map.get(emp.status_id);
      if(map_item === undefined)
      {
        const item: StatisticItem =  {
        status_name: get_status(emp.status_id ?? "")?.status ?? "Неизвестно",
        color: get_status(emp.status_id ?? "")?.color ?? "#D03050",
        logo: get_status(emp.status_id ?? "")?.logo,
        count: 1,
        percentage_str: total_employees.value > 0 ? `${((1 / total_employees.value) * 100).toFixed(1)}%` : '0%',
        percentage: total_employees.value > 0 ? Math.round((1 / total_employees.value) * 100) : 0
      }
        map.set(emp.status_id, item);
      }
      else
      {
        map_item.count += 1;
        map_item.percentage_str = total_employees.value > 0 ? `${((map_item.count / total_employees.value) * 100).toFixed(1)}%` : '0%';
        map_item.percentage = total_employees.value > 0 ? Math.round((map_item.count / total_employees.value) * 100) : 0
        map.set(emp.status_id, map_item);
      }
    }
    else
    {
      const map_item = map.get("unknown");
      if(map_item === undefined)
      {
        const item: StatisticItem =  {
        status_name: "На рабочем месте",
        color: "#30cf60",
        logo: null,
        count: 1,
        percentage_str: total_employees.value > 0 ? `${((1 / total_employees.value) * 100).toFixed(1)}%` : '0%',
        percentage: total_employees.value > 0 ? Math.round((1 / total_employees.value) * 100) : 0
      }
        map.set("unknown", item);
      }
      else
      {
        map_item.count += 1;
        map_item.percentage_str = total_employees.value > 0 ? `${((map_item.count / total_employees.value) * 100).toFixed(1)}%` : '0%';
        map_item.percentage = total_employees.value > 0 ? Math.round((map_item.count / total_employees.value) * 100) : 0;
        map.set("unknown", map_item);
      }
    }
  });

  return Array.from(map.values()).sort((a,b) => a.status_name > b.status_name ? 1 : -1);
})

</script>
  
<style scoped lang="scss">
.tags
{
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  gap: 10px;
}
</style>