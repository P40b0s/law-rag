<template lang="pug">
n-modal(
    :show="props.show" 
    preset="dialog"
    title="Выбор тега"
    type="success"
  
    @close="emits('update:show', false)"
    )
    template(#icon)
        n-icon(color="#1fe2ef" size="24"): PricetagOutline
    .tags
        n-tag.tag(v-for="tag in tags" type="success" @click="select_tag(tag)") {{tag}}
</template>

<script setup lang="ts">
//import {  } from 'vue'
import { 
    NModal,
    NIcon,
    NText,
    NTag
} from 'naive-ui'
import { PricetagOutline  } from '@vicons/ionicons5'
import { type Task } from '@/types/task'
import { ref, watch } from 'vue';
import { http_sevice } from '@/services/http_service/http_service';


interface Props
{
    show: boolean,
}

interface Emits
{
    (e: 'selected-tag', tag: string): void,
    (e: 'update:show', show: boolean): void,
}
const props = defineProps<Props>();
const emits = defineEmits<Emits>();
const tags = ref<string[]>([])
const select_tag = (tag: string) =>
{
    emits('update:show', false);
    emits('selected-tag', tag);
}
watch(() => props.show, async (n) =>
{
    if(n)
    {
        tags.value = await http_sevice.tasks_service.get_tags();
    }
})

</script>

<style scoped>
.tags
{
    display: flex;
    gap: 10px;
}
.tag:hover
{
    color: #288ed7;
    cursor: pointer;
}
</style>