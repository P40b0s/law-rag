<template lang="pug">
div.progress-container(v-if="props.stages.length > 0")
  n-space(vertical size="large")
    //- Сегментированный прогрессбар
    div.progress-bar
      div.progress-segments
        div.segment(
          v-for="(stage, index) in props.stages"
          :key="index"
          :class="get_segment_class(stage)"
          :style="{ width: segment_width + '%' }"
        )
            n-tooltip
                template(#trigger)
                    div.segment-content
                        div.task-date {{ format_date(stage.timestramp) }}
                div  {{ get_tooltip_text(stage) }}
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { 
  NAlert, 
  NSpace, 
  NButton, 
  NCheckbox, 
  NInput, 
  NForm, 
  NFormItem, 
  NDynamicInput,
  NTooltip,
  NDatePicker,
} from 'naive-ui'
import { DateFormat, DateTime } from '@/services/date'
import { type TaskStage } from '@/types/task'


interface Task 
{
  stages: TaskStage[]
}

const props = defineProps<Task>()

function format_date(timestamp: number): string 
{
    return DateTime.parse(timestamp).to_string(DateFormat.DateMonth)
}


// Вычисляемые свойства
const total_tasks = computed(() => props.stages.length)
const completed_count = computed(() => props.stages.filter(task => task.completed).length)
//const overdue_count = computed(() => tasks.value.filter(task => isOverdue(task)).length)
const progress_percentage = computed(() => 
  total_tasks.value > 0 ? Math.round((completed_count.value / total_tasks.value) * 100) : 0
)
const segment_width = computed(() => 100 / total_tasks.value)

// Методы

function is_overdue(task: TaskStage): boolean 
{
  if (task.completed) return false
  return task.timestramp < new Date().setHours(0, 0, 0, 0)
}

const get_segment_class = (stage: TaskStage) => 
{
  if (stage.completed) return 'completed'
  if (is_overdue(stage)) return 'overdue'
  return 'active'
}

const get_tooltip_text = (stage: TaskStage) => {
  const status = stage.completed ? 'Выполнено' : is_overdue(stage) ? 'ПРОСРОЧЕНО' : 'Не выполнено'
  return `${stage.name} - ${status} (до ${format_date(stage.timestramp)})`
}

</script>

<style scoped>
.progress-container {
  width: 100%;
  margin: 0 auto;
  padding: 0px;
}

.progress-bar {
  width: 100%;
  height: 20px;
  background-color: var(--n-color);
  border-radius: 8px;
  overflow: hidden;
  border: 2px solid #d9d9d9;
}

.progress-segments {
  display: flex;
  height: 100%;
  width: 100%;
}

.segment {
  height: 100%;
  transition: all 0.3s ease;
  cursor: pointer;
  position: relative;
  border-right: 2px solid white;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
}

.segment:last-child {
  border-right: none;
}

.segment:hover {
  /* transform: scaleY(1.1); */
  z-index: 1;
  box-shadow: 0 0 10px rgba(0, 0, 0, 0.2);
}

.segment.active {
  background-color: #e8e8e8;
}

.segment.completed {
  background-color: #18a058;
}

.segment.overdue {
  background-color: #d03050;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0% { opacity: 1; }
  50% { opacity: 0.8; }
  100% { opacity: 1; }
}

.segment-content {
  height: 100%;
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  color: rgba(0, 0, 0, 0.7);
  font-weight: 500;
  text-align: center;
  padding: 2px;
}

.task-date {
  font-size: 12px;
  opacity: 0.8;
  margin-top: 2px;
  color: var(--n-text);
  font-weight: 600;
  width: 100%;
}

.segment.completed .segment-content {
  color: white;
}

.segment.overdue .segment-content {
  color: white;
  font-weight: bold;
}
</style>