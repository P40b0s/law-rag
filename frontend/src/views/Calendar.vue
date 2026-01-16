<template lang="pug">
employees-calendar(
    @cell-click="handleCellClick"
    @person-select="handlePersonSelect")
</template>
    
<script lang="ts">
import { ref, type Component, watch, inject, onMounted, onUnmounted, computed, onBeforeUnmount, h, toRefs } from 'vue';
import { type Events, type Emitter } from '../services/emitter';
import { NForm, NTabs, NTabPane, NFormItem, NInput, NButton, darkTheme } from 'naive-ui';
import { notify_service } from '@/services/notification_service';
import EmployeesCalendar from '@/components/EmployeesCalendar.vue';
import { http_sevice } from '@/services/http_service/http_service';
import { type CalendarEmployee, type EmployeeState } from '@/types/employees';
</script>
<script lang="ts" async setup>
const emitter = inject<Emitter<Events>>('emitter') as Emitter<Events>;



const handleCellClick = (person: CalendarEmployee, date: Date, period: EmployeeState | null) => {
  console.log('Клик по ячейке:', person.employee.surname, date, period)
}

const handlePersonSelect = (person: CalendarEmployee) => {
  console.log('Выбран человек:', person.employee.surname)
}
</script>
    
<style lang="scss" scoped>

</style>