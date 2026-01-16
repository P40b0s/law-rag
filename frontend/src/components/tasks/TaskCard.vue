<template lang="pug">
n-card(
  size="small"
  :class="`task-card priority-${task.priority}`"
  @click="edit_task(task)"
)

  n-thing
    template(#header)
      div {{task.title}}
    template(#header-extra)
      n-button(
        text 
        @click.stop="delete_task(task.id)" 
        size="tiny"
        type="error"
        style="margin-left: 8px;"
      )
        template(#icon)
          n-icon(:size="20")
            TrashOutline
    template(#description)
      n-space(vertical :size="8")
        n-text(depth="3") {{ task.description }}
    template(#footer)
      n-space(justify="space-between" align="center")
        n-tag(size="small") {{ get_department_name(task.department_id) }}
    template(#action)
      div
        segmented-progressbar(:stages="task.task_stages")
        n-progress(:percentage="progress.progress")
          span {{progress.left}} дн.
    
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { 
  NLayout, 
  NLayoutSider, 
  NLayoutHeader, 
  NLayoutContent,
  NSpace,
  NH2,
  NH4,
  NList,
  NListItem,
  NThing,
  NDivider,
  NInput,
  NButton,
  NIcon,
  NGrid,
  NGi,
  NCard,
  NTag,
  NText,
  NModal,
  NForm,
  NFormItem,
  NSelect,
  NInputNumber,
  NDrawer,
  NDrawerContent,
  NStatistic,
  NDataTable,
  NProgress
} from 'naive-ui'
import { AddCircleOutline, Close, Time, StopCircle, PlayCircle, CheckmarkCircle,  TrashOutline, Warning  } from '@vicons/ionicons5'
import { VueDraggableNext as draggable } from 'vue-draggable-next'
import { notify_service } from '@/services/notification_service'
import {type DragChangeEvent} from '@/types/draggable_item';
import { useDictionaries } from '@/composables/useDictionaries'
import { type Task } from '@/types/task'
import { DateTime, getDaysDiff } from '@/services/date'
import SegmentedProgressbar from './SegmentedProgressbar.vue'
const {departmentOptions, get_department} = useDictionaries();

interface Props
{
  task: Task
}
interface Emits
{
  (e: 'edit', task: Task): void,
  (e: 'delete', task: Task): void,
}

const props = defineProps<Props>();
const emits = defineEmits<Emits>();
const progress = computed(() => 
{
  return getDaysDiff(props.task.added_by[1]?.as_date() as Date, props.task.target_date?.as_date() as Date)
})
const get_department_name = (department_id: string) => 
{
  return get_department(department_id)?.value ?? 'Неизвестно'
}

const edit_task = (task: Task) => 
{
  emits('edit', props.task);
}
const delete_task = (task: Task) => 
{
  emits('delete', props.task);
}

</script>

<style scoped>

.task-card {
  margin-bottom: 12px;
  cursor: grab;
}

.task-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.task-card.priority-low {
  border-left: 4px solid #d4d4d8;
}

.task-card.priority-medium {
  border-left: 4px solid #f59e0b;
}

.task-card.priority-high {
  border-left: 4px solid #b9102c;
}
</style>