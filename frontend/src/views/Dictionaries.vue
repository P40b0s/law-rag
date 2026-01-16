<template lang="pug">
n-tabs.tabs(type="line" animated )
    n-tab-pane(name="clinics" tab="Поликлинники")
        clinic-editor
    n-tab-pane(name="departments" tab="Отделы")
        department-editor
    n-tab-pane(name="statuses" tab="Статусы сотрудников")
        employee-status-editor
    n-tab-pane(name="properties" tab="Свойства сотрудников")
        property-manager
</template>
    
<script lang="ts">
import { ref, type Component, watch, inject, onMounted, onUnmounted, computed, onBeforeUnmount, h, toRefs } from 'vue';
import { type Events, type Emitter } from '../services/emitter';
import { NForm, NTabs, NTabPane, NFormItem, NInput, NButton, darkTheme } from 'naive-ui';
import { notify_service } from '@/services/notification_service';
import ClinicEditor from '@/components/ClinicEditor.vue';
import DepartmentEditor from '@/components/DepartmentEditor.vue';
import EmployeeStatusEditor from '@/components/EmployeeStatusEditor.vue';
import PropertyManager from '@/components/PropertyManager.vue';
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