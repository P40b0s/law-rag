<template lang="pug">
n-modal(
    :show="props.show" 
    preset="dialog"
    title="Подтверждение удаления"
    type="warning"
    positive-text="Удалить"
    negative-text="Отмена"
    @positive-click="confirm_delete"
    @negative-click="emits('update:show', false)"
    @close="emits('update:show', false)"
    )
    template(#icon)
        n-icon(color="#f0a020" size="24")
            Warning
    n-text Вы уверены, что хотите удалить задачу "
        strong {{ task?.title }}
        | "? Это действие нельзя отменить.
</template>

<script setup lang="ts">
//import {  } from 'vue'
import { 
    NModal,
    NIcon,
    NText
} from 'naive-ui'
import { Warning  } from '@vicons/ionicons5'
import { type Task } from '@/types/task'


interface Props
{
    task?: Task,
    show: boolean,
}

interface Emits
{
    (e: 'delete', task_id: string): void,
    (e: 'update:show', show: boolean): void,
}
const props = defineProps<Props>();
const emits = defineEmits<Emits>();
// Метод для подтверждения удаления
const confirm_delete = () => 
{
    if(props.task?.id)
        emits('delete', props.task.id);
    emits('update:show', false);
}

</script>

<style scoped>

</style>