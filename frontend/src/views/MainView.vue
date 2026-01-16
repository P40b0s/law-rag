<template lang="pug">
n-tabs.tabs(type="line" animated )
    n-tab-pane(name="clinics" tab="Список документов")
      document-loader
    n-tab-pane(name="departments" tab="Отделы")
        
    n-tab-pane(name="statuses" tab="Статусы сотрудников")
       
    n-tab-pane(name="properties" tab="Свойства сотрудников")
        
</template>
    
<script lang="ts">
import { ref, type Component, watch, inject, onMounted, onUnmounted, computed, onBeforeUnmount, h, toRefs } from 'vue';
import { type Events, type Emitter } from '../services/emitter';
import { NForm, NTabs, NTabPane, NFormItem, NInput, NButton, darkTheme } from 'naive-ui';
import { notify_service } from '@/services/notification_service';
import DocumentLoader from '../components/DocumentLoader.vue';
//import  user_service  from '../services/user_service';
</script>
<script lang="ts" setup>
const emitter = inject<Emitter<Events>>('emitter') as Emitter<Events>;
</script>
    
<style lang="scss" scoped>
.tabs
{
    display: flex;
    flex-direction: column;
    align-items: start;
}
</style>