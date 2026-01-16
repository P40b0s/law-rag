<template lang="pug">
div
  .tracker-header
    .search-panel
      n-select.department-selector(
        :options="departmentOptions" 
        v-model:value="filter.department" 
        clearable 
        placeholder="Выберите отдел")

      n-checkbox-group(v-model:value="filter.status" style="margin-left: 8px;")
        n-space(horizontal)
          n-checkbox(
            v-for="status in status_options"
            :key="status.value"
            :value="status.value"
            :label="status.label")
      n-tooltip(v-if="!filter.tag")
        template(#trigger)
          n-button.tag-selector(flat text @click="show_select_tags_dialog = true" )
            template(#icon)
              n-icon(size="30"): PricetagOutline
        span Поиск по тегам
      n-date-picker(
              v-model:value="filter.dates_range"
              type="daterange"
              placeholder="Период"
              :time-picker-props="{ timeZone: 'Europe/Moscow' }"
              clearable
            )
      n-button(@click="add_new_task" type="primary") 
        template(#icon)
            n-icon: AddCircleOutline
        span Новая задача
    .tags-search
      n-tag(v-if="filter.tag" type="warning" closable @close="remove_tag") {{filter.tag}}

  .tasks  
    task-listbox(:tasks="tasks" @edit="task_edit_request" @delete="task_delete_request")
  .loading(v-if="loading")
    loader(status="Ожидайте выполнения задачи....")
  task-edit-form(v-model:task="edited_task" v-model:show="show_edit_modal" @save="save_task")
  task-delete-dialog(:task="deleted_task" v-model:show="show_delete_modal" @delete="delete_task")
  tags-dialog(v-model:show="show_select_tags_dialog" @selected-tag="tag_selected")
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
  NTooltip,
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
  NCheckboxGroup,
  NDatePicker,
  NCheckbox
} from 'naive-ui'
import { AddCircleOutline, Close, Time, StopCircle, PlayCircle, CheckmarkCircle,  TrashOutline, Warning, Search, PricetagOutline } from '@vicons/ionicons5'
import { VueDraggableNext as draggable } from 'vue-draggable-next'
import { notify_service } from '@/services/notification_service'
import {type DragChangeEvent} from '@/types/draggable_item';
import { useDictionaries } from '@/composables/useDictionaries'
import { http_sevice } from '@/services/http_service/http_service'
import { load_from_localstorage, remove_from_localstorage, save_to_localstorage, sleepNow } from '@/services/helpers'
import { type TaskEvent, type Task, type TaskDeleteEvent, type TaskFilter } from '@/types/task'
import TaskEditForm from './TaskEditForm.vue'
import TaskDeleteDialog from './TaskDeleteDialog.vue'
import TagsDialog from './TagsDialog.vue'
import TaskCard from './TaskCard.vue'
import useUser from '@/composables/useUser'
import Loader from '../Loader.vue'
import { DateTime } from '@/services/date'
import TaskListbox from './TaskListbox.vue'
import emitter, {type Events} from '@/services/emitter'
import background from '@svg/back.svg';
const {departmentOptions} = useDictionaries();
const {get_user} = useUser();


//сделать запрос с бд по фильтам департамент, год, месяц ( где в месяц попадает целевая дата)
const tasks = ref<Task[]>([])
const edited_task = ref<Task>();
const deleted_task = ref<Task>();
const show_edit_modal = ref(false);
const show_delete_modal = ref(false);
const show_select_tags_dialog = ref(false);
const loading = ref(false);
const tag_selected = (selected_tag: string) =>
{
  filter.value.tag = selected_tag
}
const remove_tag = () =>
{
  filter.value.tag = undefined;
}
const color = computed(() =>
{
  if(filter.value.dates_range 
  || filter.value.department
  || (filter.value.status && filter.value.status.length > 0)
  || filter.value.tag)
  {
    return '#0f3539ad'
  }
  else
  {
    return 'transparent'
  }
})
const priorityOptions = [
  { label: 'Низкий', value: 'low' },
  { label: 'Средний', value: 'medium' },
  { label: 'Высокий', value: 'high' }
]
const filter = ref<TaskFilter>({
})
const status_options = computed(() =>
{
  return [
    { label: 'К выполнению', value: "todo" },
    { label: 'В работе', value: "in_progress" },
    { label: 'Выполнено', value: "done" },
  ]
})

const task_statuses = ['todo', 'in_progress', 'done'] as const
// Компьютеды
const tasks_by_status = computed(() => 
{
  const result: Record<string, Task[]> = 
  {
    todo: [],
    in_progress: [],
    done: []
  }
  tasks.value.forEach(task => 
  {
    if (result[task.status]) 
    {
      result[task.status].push(task)
    }
  })
  return result
})
const add_task_event = (event: TaskEvent) =>
{
  tasks.value.push(event.task);
}
const edit_task_event = (event: TaskEvent) =>
{
  const index = tasks.value.findIndex(f=>f.id != undefined && f.id == event.task.id);
  if(index != -1)
    tasks.value[index] = event.task;
}
const delete_task_event = (event: TaskDeleteEvent) =>
{
  const index = tasks.value.findIndex(f=>f.id == event.task_id);
  if (index != -1)
  {
    tasks.value.splice(index);
  }
}


const task_edit_request = (task: Task) =>
{
  edited_task.value = task;
  show_edit_modal.value = true;
}
const task_delete_request = (task: Task) =>
{
  deleted_task.value = task;
  show_delete_modal.value = true;
}
const save_task = async ({task, update}: {task: Task, update: boolean}) =>
{
  console.log("task saved....")
  edited_task.value = undefined;
  show_edit_modal.value = false;
  loading.value = true;
  const new_files = task.files.filter(f=>f.file);
  //загружаем файлы на сервер только если создается новая задача
  //если редактируется старая то удаление и добавление файлов производится в компоненте FileSelector
  if(update)
  {
    await http_sevice.tasks_service.edit(task)
  }
  else
  {
    await http_sevice.tasks_service.add(task)
  }
   loading.value = false;

}
const delete_task = async () =>
{
  console.log("task deleted....")
  loading.value = true;
  if(deleted_task.value?.files)
  for(const file of deleted_task.value?.files)
  {
    await http_sevice.tasks_service.delete_file(file.id);
  }
  if(deleted_task.value?.id)
    await http_sevice.tasks_service.delete(deleted_task.value?.id)
  deleted_task.value = undefined;
  loading.value = false;
}
const add_new_task = () =>
{
  show_edit_modal.value = true;
}

const get_status_title = (status: string) => {
  const titles = {
    todo: 'К выполнению',
    in_progress: 'В работе',
    done: 'Выполнено'
  }
  return titles[status as keyof typeof titles] || status
}

const get_status_tag_type = (status: string) => {
  const types = {
    todo: 'default',
    inProgress: 'warning',
    done: 'success'
  }
  return types[status as keyof typeof types] || 'default'
}


// Хуки жизненного цикла
onMounted( async () => 
{
  loading.value = true;
  const loaded_filter = load_from_localstorage('tasks-filter');
  if(loaded_filter)
    filter.value = loaded_filter;
  tasks.value = await http_sevice.tasks_service.get_filtered(filter.value)
  loading.value = false;
  emitter.on('add_task', add_task_event)
  emitter.on('edit_task', edit_task_event)
  emitter.on('delete_task', delete_task_event)
  
})
onUnmounted(() =>
{
    emitter.off('add_task', add_task_event)
    emitter.off('edit_task', edit_task_event)
    emitter.off('delete_task', delete_task_event)
})

watch(() => filter.value, async (f) =>
{
  console.log(f);
  if(!f.department
    && !f.dates_range
    && !f.status
    && !f.tag
  )
  {
    remove_from_localstorage<string>('tasks-filter')
  }
  else
  {
    save_to_localstorage('tasks-filter', f)
    loading.value = true;
    tasks.value = await http_sevice.tasks_service.get_filtered(filter.value)
    loading.value = false;
  }
}, {deep: true})

</script>

<style scoped>
.selected-department {
  background-color: #f0f9ff;
  border-radius: 6px;
  cursor: pointer;
}

.task-list {
  min-height: 100px;
}

.task-card {
  margin-bottom: 12px;
  cursor: grab;
}

.task-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.task-card.status-todo {
  border-left: 4px solid #d4d4d8;
}

.task-card.status-inProgress {
  border-left: 4px solid #f59e0b;
}

.task-card.status-done {
  border-left: 4px solid #10b981;
}

.n-thing {
  padding: 8px 12px;
  border-radius: 6px;
  transition: background-color 0.2s;
}

.n-thing:hover {
  background-color: #f8fafc;
  cursor: pointer;
}
.search-panel
{
  display: flex;
  flex-direction: row;
  width: 100%;
  align-items: center;
  justify-content: space-between;
}
.tags-search
{
  display: flex;
  justify-content: flex-start;
  align-items: center;
  justify-items: center;
  height: 100%;
}
.tag-selector
{
  margin-left: 10px;
  margin-right: 10px;
}
.tracker-header
{
  display: flex;
  flex-direction: column;
  height: 80px;
  background: v-bind(color);
  padding: 5px;
}
.department-selector
{
  flex-basis: 300px;
}
.loading
{
  position:absolute;
  width: 100vw;
  height: 100vh;
  background-color: #2c2c2c6b;
  top: 0px;
}
</style>