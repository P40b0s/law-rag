<template lang="pug">
n-card(title="Список задач" size="small")
    template(#header-extra)
      h3 {{props.tasks.length}}
    n-list.tasks(bordered hoverable)
        n-list-item(v-for="task in filtered_tasks" :key="task.id + key_date(task.changed_by)")
            task-list-item(
                :task="task"
                @edit="task_edit_request"
                @delete="task_delete_request"
            )
    
        n-list-item(v-if="filtered_tasks.length === 0")
            .empty-state
                n-empty(description="Задачи не найдены" size="small")
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { 
  NCard, 
  NList, 
  NListItem, 
  NSpace, 
  NText, 
  NDatePicker, 
  NButton, 
  NIcon,
  NEmpty
} from 'naive-ui'
import { Close } from '@vicons/ionicons5'
import { type Task } from '@/types/task'
import TaskListItem from './TaskListItem.vue'
import { DateFormat, DateTime } from '@/services/date'

interface Props {
  tasks: Task[]
}

interface Emits {
  (e: 'edit', task: Task): void
  (e: 'delete', task: Task): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const selected_date = ref<number | null>(null)
const date_filter_enabled = ref(false)

const key_date = (changed?: [string, DateTime | null]) =>
{
  if(changed && changed[1])
  {
    return changed[1].to_string(DateFormat.SerializedDateTime);
  }
  return ""
}
// Фильтрация задач по дате
const filtered_tasks = computed(() => {
  if (!date_filter_enabled.value || !selected_date.value) {
    return props.tasks
  }

  const target_date = new Date(selected_date.value)
  
  return props.tasks.filter(task => {
    if (!task.target_date) return false
    
    const task_date = new Date(task.target_date.as_date())
    return task_date.toDateString() === target_date.toDateString()
  })
})

const apply_date_filter = () => {
  date_filter_enabled.value = true
}

const clear_date_filter = () => {
  selected_date.value = null
  date_filter_enabled.value = false
}

const task_edit_request = (task: Task) => {
  emit('edit', task)
}

const task_delete_request = (task: Task) => {
  emit('delete', task)
}

// Сбрасываем фильтр при изменении исходного списка задач
watch(() => props.tasks, () => 
{
  if (date_filter_enabled.value && filtered_tasks.value.length === 0) {
    clear_date_filter()
  }
})
</script>

<style scoped>
.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 20px 0;
}
.tasks
{
  min-height: calc(100vh - 200px)
}
</style>