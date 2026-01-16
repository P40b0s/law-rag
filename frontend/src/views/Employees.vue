<template lang="pug">
.employees-main
    .employees-panel
        employees-manager(@selected="select_employee_emit" ref="employeesRef")
    .properies-panel
    n-tabs.pane(v-if="selected_employee" type="line" animated)
        n-tab-pane(name="state" tab="Состояния")
            employee-state-manager(v-model:employee="selected_employee" @saved="state_save_handle" @deleted="state_delete_handle")
        n-tab-pane(name="info" tab="Информация")
            employee-information-manager(v-model:employee="selected_employee")
</template>
    
<script lang="ts">
import { ref, type Component, watch, inject, onMounted, onUnmounted, computed, onBeforeUnmount, h, toRefs } from 'vue';
import { type Events, type Emitter } from '../services/emitter';
import { NForm, NTabs, NTabPane, NFormItem, NInput, NButton, darkTheme } from 'naive-ui';
import { notify_service } from '@/services/notification_service';
import EmployeesManager from '@/components/EmployeesManager.vue';
import EmployeeStateManager from '@/components/EmployeeStateManager.vue';
import EmployeeInformationManager from '@/components/EmployeeInformationManager.vue';
import { type EmployeeState, type Employee } from '@/types/employees';
import { DateTime } from '@/services/date';
//import  user_service  from '../services/user_service';
</script>
<script lang="ts" setup>
const emitter = inject<Emitter<Events>>('emitter') as Emitter<Events>;
const employeesRef = ref<InstanceType<typeof EmployeesManager> | null>(null);
const selected_employee = ref<Employee|null>(null);
const select_employee_emit = (emp: Employee) =>
{
    selected_employee.value = emp;
}
const state_save_handle = (emp: EmployeeState) =>
{
    employeesRef.value?.update_status(emp.employee_id, emp.id, emp.status_id, emp.start_date as DateTime, emp.end_date as DateTime)
}

const state_delete_handle = (id: string) =>
{
    employeesRef.value?.delete_status(id)
}
//employeesRef.value?.update_status()
</script>
    
<style lang="scss" scoped>
.employees-main
{
    display:  flex;
    flex-direction: row;
    gap: 10px;
    margin-right: 5px;
    margin-top: 10px;
    width: 100%;
}
.employees-panel
{
    max-width: 800px;
    width: 800px;
}
.properties-panel
{
    display:  flex;
    flex-direction: column;
}
.pane
{
    //width: 100%;
    //display: flex;
    //flex-direction: column;
    
    align-items: start;
}
</style>