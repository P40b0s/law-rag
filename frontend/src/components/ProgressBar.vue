<template lang="pug">
n-popover(v-if="props.employees" @update:show="handle_tooltip_show" trigger="hover" style="width: 500px")
  template(#trigger)
    n-badge(value="?" type="info" :show="badge_show")
      .progress-container
        n-progress.pg(v-bind="progress_props")
        .value-overlay {{ props.current_count }} ({{current_progress}}%)
  employees-list(:fios="props.employees")
    
.progress-container(v-else)
  n-progress.pg(v-bind="progress_props")
  .value-overlay {{ props.current_count }} ({{current_progress}}%)
</template>

<script lang="ts">
import { ref, computed, onMounted, watch, h } from 'vue'
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
  NTooltip,
  NBadge,
  NPopover
} from 'naive-ui'
import { type Fio } from '@/types/statistic';
import EmployeesList from './EmployeesList.vue';

</script>
<script lang="ts" setup>
const props = defineProps<{
  current_count: number,
  total_count: number,
  employees?: Fio[],
  category_name?: string,
  department_name?: string
}>()


const get_percentage = (count: number, total: number) => 
{
  return  total > 0 ? Math.round((count / total) * 100) : 0;
}
const current_progress = computed(() => get_percentage(props.current_count, props.total_count));
const badge_show = ref(true);

const handle_tooltip_show = (show: boolean) =>
{
  badge_show.value = !show;
}

//почему то не отображается value-overlay...
const RenderProgress = 
{
  setup() 
  {
    return () => h('div', { class: 'progress-container' }, 
    [
      h(NProgress, 
      {
        class: 'pg',
        percentage: current_progress.value,
        showIndicator: false,
        status: 'info',
        height: 30,
        borderRadius: 2,
        fillBorderRadius: 0,
        color: 
        {
          stops: ['#5bd9e6', '#1ea4b3']
        },
        railColor: "#a6bee894"
      }),
      h('div', {  },
        `${props.current_count} (${current_progress.value}%)`
      )
    ])
  }
}

const progress_props = computed(() => ({
  percentage: current_progress.value,
  showIndicator: false,
  height: 30,
  borderRadius: 2,
  fillBorderRadius: 0,
  color: {
    stops: ['#5bd9e6', '#1ea4b3']
  },
  railColor: "#a6bee894"
}))

</script>
  
<style lang="scss" scoped>

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
.value-overlay 
{
  position: relative;
  left: 0px;
  z-index: 20;
  font-weight: 600;
  font-size: 20px !important;
  color: #111111;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
  pointer-events: none;
  padding: 0 8px;
}

/* Адаптивность */
@media (max-width: 550px) 
{
  :deep(.progress-container) 
  {
    height: 24px;
  }
  
  :deep(.value-overlay) 
  {
    font-size: 6px !important;
    padding: 0 4px;
  }
}

@media (max-width: 768px) 
{
  :deep(.progress-container) 
  {
    height: 24px;
  }
  
  :deep(.value-overlay) 
  {
    font-size: 11px !important;
    padding: 0 4px;
  }
}

@media (max-width: 1000px) 
{
  :deep(.progress-container) 
  {
    height: 24px;
  }
  
  :deep(.value-overlay) 
  {
    font-size: 13px !important;
    padding: 0 4px;
    top: 20%;
  }
}
</style>