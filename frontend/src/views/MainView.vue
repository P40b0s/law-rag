<template lang="pug">
model-state
n-tabs.tabs(type="line" animated )
    n-tab-pane(name="documents" tab="Документы")
        documents-list
    n-tab-pane(name="collections" tab="Коллекции")
        collections
    n-tab-pane(name="queries" tab="Запросы")
        queries
    n-tab-pane(name="generation" tab="Генерация")
        generation

status
</template>
    
<script lang="ts">
import { inject, onMounted } from 'vue';
import { type Events, type Emitter } from '../services/emitter';
import { NTabs, NTabPane } from 'naive-ui';
import ModelState from '@/components/ModelState.vue';
import DocumentsList from '@/components/DocumentsList.vue';
import Collections from '@/components/Collections.vue';
import Status from '@/components/Status.vue';
import { http_service } from '@/services/http_service/http_service';
import Queries from '@/components/QueryForm.vue';
import Generation from '@/components/GenerationForm.vue';
</script>
<script lang="ts" setup>
const emitter = inject<Emitter<Events>>('emitter') as Emitter<Events>;
onMounted(async () => {
    await http_service.model_state_service.get_state()
})
</script>
    
<style lang="scss" scoped>
.tabs
{
    display: flex;
    flex-direction: column;
    align-items: start;
    padding-bottom: 60px; // Добавляем отступ снизу для панели статуса
}
</style>