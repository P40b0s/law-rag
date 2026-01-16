<template lang="pug">
n-thing(:description="props.task.description" :title="props.task.title")
  template(#avatar)
    div.avatar-template
      n-tooltip Назначеный отдел
        template(#trigger)
          n-tag(size="small") {{ get_department_name(props.task.department_id) }}
      n-tooltip Приоритет задачи
        template(#trigger)
          n-tag(size="small" :type="get_priority_type(props.task.priority)") 
            div {{ get_priority_label(props.task.priority) }}
      n-tooltip Текущий статус задачи
        template(#trigger)
          n-tag(size="small" :type="get_status_type(task.status)")
            div {{ get_status_label(props.task.status) }}
      n-tooltip Контрольная дата
        template(#trigger)
          n-tag(
            v-if="props.task.target_date"
            size="small"
            :type="is_overdue ? 'error' : 'info'")
            template(#icon)
              n-icon(size="14" style="margin-right: 4px;" :component="Time")
            span {{ format_target_date(props.task.target_date) }}
  
  template(#header)
    .task-header
      n-text(strong) {{ props.task.title }}
  template(#header-extra)
    .extra-buttons
      n-tag(v-if="is_overdue" type="error") Просрочено
      n-tooltip Редактировать
        template(#trigger)
          n-button(
              text 
              @click.stop="edit_task(task.id)" 
              size="tiny"
              type="success"
            )
            template(#icon)
              n-icon(:size="25")
                Edit
      n-tooltip Удалить
        template(#trigger)
          n-button(
              text 
              @click.stop="delete_task(task.id)" 
              size="tiny"
              type="error"
            )
            template(#icon)
              n-icon(:size="25")
                TrashOutline
  template(#footer)
    div
      segmented-progressbar(:stages="props.task.task_stages")
      n-progress(v-if="progress.left >=0" :percentage="progress.progress" :color="progress_color(progress.progress)")
        span {{progress.left}} дн.
  template(#action)
    .actions-space
      .actions-buttons
        .tags-content
          n-tag(type="success" v-for="tag in props.task.tags") {{tag}}
        

      .task-meta
        n-text(depth="3" style="font-size: 12px;") 
          n-tooltip(trigger="hover" @update:show="handle_added_update_show") {{added_by}}
            template(#trigger)
              div Создана: {{ format_date(props.task.added_by[1]) }}
        
        n-text(depth="3" style="font-size: 12px;" v-if="props.task.changed_by && props.task.changed_by.length > 1")
          n-tooltip(trigger="hover" @update:show="handle_edited_update_show") {{changed_by}}
            template(#trigger)
              div Изменена: {{ format_date(props.task.changed_by[1]) }}

  .fio-tags
    n-tag(type="success" v-for="employee in employees") {{employee}}
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { 
  NThing, 
  NTag, 
  NText, 
  NSpace, 
  NButton, 
  NIcon,
  NTooltip,
  NProgress
} from 'naive-ui'
import { Time, TrashOutline, Settings as Edit } from '@vicons/ionicons5'
import { type Task } from '@/types/task'
import { DateFormat, DateTime, dateTimeToString, getDaysDiff } from '@/services/date'
import { useDictionaries } from '@/composables/useDictionaries'
import SegmentedProgressbar from './SegmentedProgressbar.vue'
import { P, match } from 'ts-pattern'
import { http_sevice } from '@/services/http_service/http_service'
const {departmentOptions, get_department} = useDictionaries();
interface Props {
  task: Task
}

interface Emits {
  (e: 'edit', task: Task): void
  (e: 'delete', task: Task): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()
const changed_by = ref<string|undefined>();
const added_by = ref<string|undefined>();
const employees = ref<string[]>([]);
const progress = computed(() => 
{
  return getDaysDiff(props.task.added_by[1]?.as_date() as Date, props.task.target_date?.as_date() as Date)
})
const progress_color = (percentage: number) =>
{
  if(percentage < 30)
  {
    return { stops: ['##9dd7a2', '#70cd77'] }
  }
  if(percentage > 30 && percentage < 60)
  {
    return { stops: ['#68ca70', '#2bca38'] }
  }
  if(percentage > 60 && percentage < 90)
  {
    return { stops: ['#cfd076', '#dc8556'] }
  }
  if(percentage > 90)
  {
    return { stops: ['#d98181', '#e42525'] }
  }
};
// Проверка просроченности задачи
const is_overdue = computed(() => {
  if (!props.task.target_date) return false
  if (new Date(props.task.target_date.as_date()) < new Date()
      && props.task.status != 'done')
    {
      return true;
    }
    else return false
})

const get_user = async (user_id?: string) =>
{
  if(user_id)
  {
    let user = await http_sevice.user_service.get_user(user_id);
    if (user)
    {
      return `${user.surname} ${user.first_name} ${user.second_name}`
    }
  }
}

const get_employee = async (employee_id?: string) =>
{
  if(employee_id)
  {
    let employee = await http_sevice.employees_service.get_employee(employee_id);
    if (employee)
    {
      return `${employee.surname} ${employee.first_name} ${employee.second_name}`
    }
  }
}
const handle_edited_update_show = async (show: boolean) => 
{
  if(!changed_by.value && show)
  {
    if(props.task.changed_by)
    {
      const user = await get_user(props.task.changed_by[0]);
      changed_by.value = user;
    }
   
  }
}

const handle_added_update_show = async (show: boolean) => 
{
  if(!added_by.value && show)
  {
    if(props.task.added_by)
    {
      const user = await get_user(props.task.added_by[0]);
      added_by.value = user;
    }
   
  }
}
const get_priority_type = (priority?: string) => 
{
  const types: Record<string, string> = 
  {
    low: 'default',
    medium: 'warning',
    high: 'error'
  }
  return types[priority || 'medium'] || 'default'
}
const get_department_name = (department_id: string) => 
{
  return get_department(department_id)?.value ?? 'Не определен'
}

const get_priority_label = (priority?: string) => {
  const labels: Record<string, string> = {
    low: 'Низкий',
    medium: 'Средний',
    high: 'Высокий'
  }
  return labels[priority || 'medium'] || 'Средний'
}

const get_status_type = (status: string) => {
  const types: Record<string, string> = {
    todo: 'default',
    in_progress: 'warning',
    done: 'success'
  }
  return types[status] || 'default'
}

const get_status_label = (status: string) => {
  const labels: Record<string, string> = {
    todo: 'К выполнению',
    in_progress: 'В работе',
    done: 'Выполнено'
  }
  return labels[status] || status
}

const format_target_date = (date?: DateTime) => {
  if(date)
    return date.to_string(DateFormat.DotDate)
  else ""
}

const format_date = (date?: DateTime) => {
  if(date)
    return date.to_string(DateFormat.DateTime)
  else
  //return dateTimeToString(new Date(date))
  return ""
}

const edit_task = () => {
  emit('edit', props.task)
}

const delete_task = () => {
  emit('delete', props.task)
}
watch(props.task, async (n) =>
{
  n.users.forEach(async u=>
    {
      const user = await get_employee(u);
      if(user)
      {
        employees.value.push(user);
      }
    }
  )
}, {immediate: true})
</script>

<style scoped>
.avatar-template
{
  display: flex;
  flex-direction: column;
  gap: 5px;
}
.task-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  width: 100%;
}

.task-meta {
  display: flex;
  flex-direction: column;
}
.tags-content
{
  display: flex;
  gap:10px;
}
.fio-tags
{
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  width: 100%;
  gap:10px;
}
.actions-space
{
  display: flex;
  width: 100%;
  justify-content: space-between;
  gap: 10px;
}
.actions-buttons
{
  display: flex;
  align-items: center;
  gap: 10px;
}
.extra-buttons
{
  display: flex;
  width: 100%;
 
}
.extra-buttons > *
{
  margin-right: 20px;
}

.n-thing {
  width: 100%;
  padding: 8px 0;
}
</style>