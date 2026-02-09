<template lang="pug">
model-state
n-tabs.tabs(type="line" animated )
    n-tab-pane(name="documents" tab="Документы")
        documents-manager
        documents-list
    n-tab-pane(name="queries" tab="Запросы")

status
</template>
    
<script lang="ts">
import { ref, type Component, watch, inject, onMounted, onUnmounted, computed, onBeforeUnmount, h, toRefs } from 'vue';
import { type Events, type Emitter } from '../services/emitter';
import { NForm, NTabs, NTabPane, NFormItem, NInput, NButton, darkTheme } from 'naive-ui';
import { notify_service } from '@/services/notification_service';
import ModelState from '@/components/ModelState.vue';
import DocumentsManager from '@/components/DocumentsManager.vue';
import DocumentsList from '@/components/DocumentsList.vue';
import Status from '@/components/Status.vue';
import { http_sevice } from '@/services/http_service/http_service';
//import  user_service  from '../services/user_service';
</script>
<script lang="ts" setup>
const emitter = inject<Emitter<Events>>('emitter') as Emitter<Events>;
onMounted(async () => {
    await http_sevice.model_state_service.get_state()
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