<template lang="pug">
n-modal(:show="props.show" preset="card" :title="editing_task ? 'Редактировать задачу' : 'Новая задача'" style="width: 600px; min-height: 800px" closable @close="cancel_handle")
    n-tabs(type="line" animated)
        n-tab-pane(name="info" tab="Карточка задачи" display-directive="show")
            n-form(:model="task_form" :rules="task_rules" ref="task_form_ref")
                n-grid(:cols="3" :x-gap="24")
                    n-gi(span="3")
                        n-form-item(label="Название" path="title")
                            n-input(v-model:value="task_form.title" placeholder="Введите название задачи")
                    
                    n-gi(span="3")
                        n-form-item(label="Описание" path="description")
                            n-input(
                            v-model:value="task_form.description"
                            type="textarea"
                            placeholder="Введите описание задачи"
                            :rows="3"
                            )

                    n-gi
                        n-form-item(label="Контрольная дата" path="target_date")
                            n-date-picker(
                                v-model:value="task_form.target_date"
                                format="dd.MM.yyyy"
                                type="date")

                    n-gi
                        n-form-item(label="Приоритет" path="priority")
                            n-select(
                            v-model:value="task_form.priority"
                            :options="priority_options"
                            placeholder="Выберите приоритет")

                    n-gi
                        n-form-item(label="Статус" path="status")
                            n-select(
                            v-model:value="task_form.status"
                            :options="statuses_options"
                            placeholder="Выберите статус")

                    
                    n-gi(span="3")
                        n-form-item(label="Отдел" path="department_id")
                            n-select(
                            v-model:value="task_form.department_id"
                            :options="departmentOptions"
                            placeholder="Выберите отвественный отдел"
                            )

                    n-gi(span="3")
                        n-form-item(label="Отвественные сотрудники" path="users")
                            n-select(
                            v-model:value="task_form.users"
                            :options="users_options"
                            multiple
                            :render-tag="render_tag"
                            placeholder="Выберите сотрудников")

                    n-gi(span="3")
                        n-form-item(label="Заметка" path="note")
                            n-input(
                            v-model:value="task_form.note"
                            type="textarea"
                            placeholder="Введите заметку при необходимости"
                            :rows="3")
                    n-gi(span="3")
                        n-form-item(label="Теги" path="tags")
                            n-dynamic-tags(v-model:value="task_form.tags")
                                template(#input="{ submit, deactivate }")
                                    n-select(
                                        ref="auto_complete_inst_ref"
                                        v-model:value="tag_input_value"
                                        size="medium"
                                        filterable
                                        tag
                                        :options="tags_options"
                                        @update:value="(v) => {submit(v); tag_selected()}"
                                        @blur="deactivate")
                                    
                                template(#trigger="{ activate, disabled }")
                                    n-button(
                                        size="small"
                                        type="primary"
                                        dashed
                                        :disabled="disabled"
                                        @click="activate()")
                                
                                        template(#icon)
                                            n-icon
                                                Add
                                        div Добавить тег
                n-space(justify="end")
                    n-button(@click="cancel_handle") Отмена
                    n-button(
                    @click="save_task"
                    type="primary"
                    ) {{ editing_task ? 'Обновить' : 'Создать' }}

        n-tab-pane(name="stages" tab="Стадии выполнения" display-directive="show" v-if="task_form.target_date")
            n-card
                n-dynamic-input(
                v-model:value="task_form.task_stages"
                :min="1"
                :max="10"
                show-sort-button
                @create="handle_create_stage"
                @remove="handle_remove_stage")

                    template(#default="{ value, index }")
                        .stage
                            n-input(
                                v-model:value="value.name"
                                type="textarea"
                                placeholder="Введите названии стадии"
                                @update:value="update_task_stage(index, $event)")
                            .date
                                n-date-picker(
                                    v-model:value="value.timestramp"
                                    type="date"
                                    format="dd.MM.yyyy"
                                    placeholder="Выберите дату"
                                    :is-date-disabled="disable_previous_dates"
                                    @update:value="update_stage_date(index, $event)")

                                n-checkbox(
                                    v-model:checked="value.completed"
                                    @update:value="update_stage_completion(index, $event)") Выполнено

        n-tab-pane(name="files" :tab="`Файлы (${files_count})`" display-directive="show" :disabled="!props.task")
            file-selector(
                v-model:files="task_form.files"
                :task_id="props.task?.id"
                :max_files="50"
                :max_file_size_MB="200"
                
                @files-change="handle_files_change")

</template>
<!-- accept=".jpg,.jpeg,.png,.pdf,.doc,.docx,.txt,.zip,.rar" -->
<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, h, nextTick } from 'vue'
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
    NTabs,
    NTabPane,
    NDynamicTags,
    NAutoComplete,
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
    NDatePicker,
    NCheckbox,
    NDynamicInput,
    type FormInst,
    type SelectOption,
    type SelectGroupOption,
    type SelectRenderTag,
    type AutoCompleteInst
} from 'naive-ui'
import { AddCircleOutline, Close, Time, StopCircle, PlayCircle, CheckmarkCircle, TrashOutline, Warning, Add } from '@vicons/ionicons5'
import { VueDraggableNext as draggable } from 'vue-draggable-next'
import { notify_service } from '@/services/notification_service'
import { type DragChangeEvent } from '@/types/draggable_item';
import { type TaskStage, type Task, type TaskFile } from '@/types/task'
import { useDictionaries } from '@/composables/useDictionaries'
import { DateFormat, DateTime } from '@/services/date'
import useUser from '@/composables/useUser'
import { type AssotiatedEmployee } from '@/types/employees'
import { http_sevice } from '@/services/http_service/http_service'
import FileSelector from './FileSelector.vue'
import { sleepNow } from '@/services/helpers'
import { date_str } from '../../services/helpers';
const {departmentOptions, get_department} = useDictionaries();
const {get_user} = useUser();
interface Props {
    //если передется задача, то форма будет для редактирования
    task?: Task,
    show: boolean,
}
interface Emits {
    //сохранение новой задачи или обновление существующей
    (e: 'save', task: {task: Task, update: boolean}): void,
    (e: 'update:task', task: Task|undefined): void,
    (e: 'update:show', show: boolean): void,
}

const props = defineProps<Props>();
const emits = defineEmits<Emits>();

const editing_task = ref<Task | null>(null);
const form_ref = ref<FormInst | null>(null)
const users_options = ref<Array<SelectOption | SelectGroupOption>>([]);
const show_files = ref(true);
const files_count = computed(() => task_form.value.files.length)
const tags = ref<string[]>([]);
const tags_options = computed(() => 
    {
        const t = tags.value.filter(f=>!task_form.value.tags.includes(f)).map(t=>
        {

            return {
                label: t,
                value: t
            }
        })
        t.push({
            label: tag_input_value.value ?? "",
            value: tag_input_value.value ?? ""
        });
        return t;
    }
)

const auto_complete_inst_ref = ref<AutoCompleteInst | null>(null)
const tag_input_value = ref<string|null>(null)
const tag_selected = () =>
{
    tag_input_value.value = "";
}
watch(auto_complete_inst_ref, (value) => 
{
  if (value)
    nextTick(() => value.focus())
})

// Форма задачи
const task_form = ref({
    title: '',
    description: '',
    priority: 'medium' as 'low' | 'medium' | 'high',
    department_id: undefined as string | undefined,
    users: [] as Array<string>,
    status: 'todo' as 'todo' | 'in_progress' | 'done',
    target_date: null as number | null,
    note: '' as string,
    files: [] as TaskFile[],
    tags: [] as string[],
    task_stages: [] as TaskStage[],
})

// Валидация формы
const task_rules =
{
    title: { required: true, message: 'Введите название задачи', trigger: 'blur' },
    departmentId: { required: true, message: 'Выберите отдел', trigger: 'change' }
}


const priority_options = [
    { label: 'Низкий', value: 'low' },
    { label: 'Средний', value: 'medium' },
    { label: 'Высокий', value: 'high' }
]

const task_statuses = ['todo', 'in_progress', 'done'] as const
const statuses_options = [
    { label: 'К выполнению', value: 'todo' },
    { label: 'В работе', value: 'in_progress' },
    { label: 'Выполнено', value: 'done' }
]

const edit_task = (task: Task) => 
{
    editing_task.value = task;
    
    task_form.value = {
        title: task.title,
        description: task.description,
        priority: task.priority,
        department_id: task.department_id,
        users: task.users,
        status: task.status,
        target_date: task.target_date?.as_date().getTime() ?? null,
        note: task.note,
        files: task.files,
        tags: task.tags,
        task_stages: task.task_stages
    }
}
const cancel_handle = async () =>
{
    emits('update:show', false);
    await sleepNow(200)
    reset_task_form();
    emits('update:task', undefined);
}

const save_task = async () => 
{
    const errors = await form_ref.value?.validate();
    if (errors?.warnings) 
    {
        notify_service.notify_error('Пожалуйста, исправьте ошибки в форме', '')
        console.error("Ошибки при валидации данных ", errors.warnings);
        return false
    }
    else
    {
        const current_user_id = get_user().value?.id;
        if(current_user_id)
        {
            if(editing_task.value)
            {
                emits('save', 
                {
                    task: 
                    {
                        ...editing_task.value,
                        ...task_form.value,
                        //мы валидируем форму поэтому эти поля не могут быть null
                        target_date: DateTime.parse(task_form.value.target_date as number),
                        changed_by: [current_user_id, DateTime.new()],
                    },
                    update: true
                })
                editing_task.value = null;
            }
            else
            {
                emits('save', 
                {
                    task: 
                    {
                        //ID будет получен при создании задачи на сервере,
                        ...task_form.value,
                        status: 'todo',
                        target_date: DateTime.parse(task_form.value.target_date as number),
                        added_by: [current_user_id, DateTime.new()],
                    },
                    update: false
                })
            }
            emits('update:show', false);
            await sleepNow(200)
            reset_task_form();
            emits('update:task', undefined);
            
        }
        else
        {
            notify_service.notify_error("Не могу определить текущего пользователя, редактирование задач невозможно!")
        }
    }
}

const handle_files_change = async (files: TaskFile[]) =>
{
    // if(props.task?.id)
    // {
    //     const files = await http_sevice.tasks_service.get_files(props.task?.id)
    //     task_form.value.files = files;
    // }
    
}

const reset_task_form = () => 
{
    task_form.value = {
        title: '',
        description: '',
        priority: 'medium',
        department_id: undefined,
        users: [],
        status: 'todo',
        target_date: null,
        note: '',
        files: [],
        tags: [],
        task_stages: []
    }
    editing_task.value = null
}

const get_initials = (emp: AssotiatedEmployee) =>
{
    return `${emp?.surname} ${emp?.first_name[0]}.${emp?.second_name[0]}.`
}
const load_users = async () =>
{
    tags.value = await http_sevice.tasks_service.get_tags();
    const list = await http_sevice.employees_service.get_employees_list();
    if(list)
    {
        let hm: Map<string, {label: string, value: string}[]> = new Map();
        list.forEach(m=> 
        {
            if(m)
            {
                let map = hm.get(m.department);
                if(!map)
                {
                    hm.set(m.department, [{label: get_initials(m), value: m.id}])
                }
                else
                {
                    map.push({label: get_initials(m), value: m.id})
                }
            }
            
        });
        for(const [k, v] of hm)
        {
            const dep = get_department(k);
            users_options.value.push(
                {
                    type: 'group',
                    label: dep?.value,
                    key: k,
                    children: v
                }
            )
        }
    }
    
}

const render_tag: SelectRenderTag = ({ option, handleClose }) => 
{
  return h(
    NTag,
    {
      type: 'success' as 'success' | 'warning' | 'error',
      closable: true,
      onMousedown: (e: FocusEvent) => 
      {
        e.preventDefault()
      },
      onClose: (e: MouseEvent) => 
      {
        e.stopPropagation()
        handleClose()
      }
    },
    { default: () => option.label }
  )
}
// Хуки жизненного цикла
onMounted(async () => 
{
    await load_users();
    // if(props.task) 
    // {
    //    edit_task(props.task);
    // }
    // console.log("форма редактирования смонтирована!")
})

watch(() => props.task, (n) =>
{
    if(props.task) 
    {
        console.log("задача на редактировании!")
       edit_task(props.task);
    }
})






function disable_previous_dates(timestamp: number) 
{
  const prev = timestamp < new Date().setHours(0, 0, 0, 0)
  if(task_form.value.target_date)
  {
    const next = timestamp > new Date(task_form.value.target_date).getTime();
    return prev || next;
  }
  else return prev
}

// Вспомогательные функции для работы с датами
function get_timestamp(days_offset: number): number 
{
  const date = new Date()
  date.setDate(date.getDate() + days_offset)
  date.setHours(0, 0, 0, 0)
  return date.getTime()
}


const handle_create_stage = (): TaskStage => 
{
  const tomorrow = get_timestamp(1);
  const tomorrow_is_disabled = disable_previous_dates(tomorrow);
  if(task_form.value.task_stages.length > 0)
  {
    const last = new Date(task_form.value.task_stages[task_form.value.task_stages.length-1].timestramp);
    last.setDate(last.getDate() + 1);
    last.setHours(0, 0, 0, 0)
    if(!disable_previous_dates(last.getTime()))
        return { name: '', completed: false, timestramp: last.getTime() }
    else  return { name: '', completed: false, timestramp: tomorrow_is_disabled ? get_timestamp(0) : tomorrow }
  }
  else
  return { name: '', completed: false, timestramp: tomorrow_is_disabled ? get_timestamp(0) : tomorrow }
}

const handle_remove_stage = (index: number) => 
{
  //message.info(`Задача "${tasks.value[index].task}" удалена`)
}


const update_task_stage = (index: number, value: string) => 
{
  task_form.value.task_stages[index].name = value
}

const update_stage_date = (index: number, value: number) => 
{
  task_form.value.task_stages[index].timestramp = value
}

const update_stage_completion = (index: number, value: boolean) => 
{
  task_form.value.task_stages[index].completed = value
}

const add_task_stage = () => 
{
  if (task_form.value.task_stages.length < 10) 
  {
    const tomorrow = get_timestamp(1)
    task_form.value.task_stages.push({ name: 'Новая задача', completed: false, timestramp: tomorrow })
    //message.success('Новая задача добавлена')
  } 
  else 
  {
    //message.warning('Максимальное количество задач - 10')
  }
}


const clear_completed_stages = () => 
{
  const completed_tasks = task_form.value.task_stages.filter(task => task.completed)
  task_form.value.task_stages = task_form.value.task_stages.filter(task => !task.completed)
  //message.success(`Удалено ${completedTasks.length} выполненных задач`)
}

const reset_all_stages = () => 
{
  task_form.value.task_stages = task_form.value.task_stages.map(task => ({ ...task, completed: false }))
  //message.info('Все задачи сброшены')
}
function is_overdue(task: TaskStage): boolean 
{
  if (task.completed) return false
  return task.timestramp < new Date().setHours(0, 0, 0, 0)
}

const mark_overdue_as_urgent = () => 
{
  const overdue_tasks = task_form.value.task_stages.filter(task => is_overdue(task))
  if (overdue_tasks.length === 0) 
  {
    //message.info('Нет просроченных задач')
  } 
  else 
  {
    //message.warning(`Найдено ${overdueTasks.length} просроченных задач!`)
  }
}


</script>

<style scoped>

.stage
{
    display: flex;
    flex-direction: column;
    width: 100%;
    gap: 10px;
}
.date
{
    display: flex;
    flex-direction: row;
    gap: 10px;
    justify-content: space-between;
    align-items: center;
}
</style>