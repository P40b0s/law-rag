<template lang="pug">
.n-dashboard
  .dashboard-header
    .header-controls
      n-button(@click="toggleEditMode" type="primary" size="small" v-if="widgetsLayout.length > 0") 
        | {{ isEditMode ? 'Сохранить' : 'Редактировать' }}
      n-button.add-widget-btn(
        @click="showAddWidgetModal = true",
        type="primary",
        circle,
        size="large"
      ) +
  
  .dashboard-grid(ref="gridEl")
    grid-layout(
      :layout.sync="widgetsLayout",
      :col-num="12",
      :row-height="30",
      :is-draggable="isEditMode",
      :is-resizable="isEditMode",
      :vertical-compact="true",
      :use-css-transforms="true",
      :margin="[10, 10]",
      @layout-updated="onLayoutUpdated"
    )
      grid-item(
        v-for="widget in widgetsLayout",
        :key="widget.i",
        :x="widget.x",
        :y="widget.y",
        :w="widget.w",
        :h="widget.h",
        :i="widget.i"
      )
        .widget(
          :class="{ 'widget-editing': isEditMode }",
          @dblclick="onWidgetDoubleClick(widget)"
        )
          .widget-header
            .widget-controls(v-if="isEditMode")
              .widget-title {{ getWidgetTitle(widget.i) }}
              n-button(
                @click="removeWidget(widget.i)",
                type="error",
                size="tiny",
                ghost
              ) ×
          .widget-content
            suspense
              component(
                :is="getWidgetComponent(widget.i)",
                :widget-id="widget.i",
                :config="getWidgetConfig(widget.i)"
              )
  
  n-modal(v-model:show="showAddWidgetModal", preset="card", title="Добавить виджет", style="width: 600px")
    .add-widget-modal
      h3 Доступные виджеты
      .widgets-list
        .widget-item(
          v-for="(widget, key) in availableWidgets",
          :key="key",
          @click="addWidget(key)"
        )
          .widget-icon {{ widget.icon }}
          .widget-name {{ widget.name }}
          .widget-desc {{ widget.description }}

  
</template>

<script lang="ts">
import { ref, type Component, watch, inject, onMounted, onUnmounted, computed, onBeforeUnmount, h, toRefs, defineComponent } from 'vue';
import { type Events, type Emitter } from '../services/emitter';
import { NForm, NFormItem, NInput, NButton, darkTheme, NModal } from 'naive-ui';
import { notify_service } from '@/services/notification_service';
import { GridLayout, GridItem } from 'vue3-grid-layout-next';
import 'vue3-grid-layout-next/dist/style.css'
import SimpleStatistic from '@/components/SimpleStatistic.vue';
import { match } from 'ts-pattern';
import { http_sevice } from '@/services/http_service/http_service';
import { type Employees } from '@/types/employees';
import EmployeesStatistic from '@/components/EmployeesStatistic.vue';
import { time_warnings, TimeWarning } from '@/components/notify_timer/use_time_warnings';
import { TimeWarningsViewer } from '@/components/notify_timer/notify_timer_viewer';
import NotifyViewer from '@/components/notify_timer/NotifyViewer.vue';
import { load_from_localstorage, save_to_localstorage } from '@/services/helpers';


// Пример компонентов-виджетов
const WidgetClock = defineComponent({
  props: ['widgetId', 'config'],
  setup(props) {
    const time = ref(new Date());
    
    const interval = setInterval(() => {
      time.value = new Date();
    }, 1000);

    onUnmounted(() => {
      clearInterval(interval);
    });

    return () => h('div', { class: 'clock-widget' }, 

    [
        h('div', { class: 'time' }, time.value.toLocaleTimeString()),
        h('div', { class: 'created' }, props.config.createdAt)
    ]
    );
  }
});

const WidgetStats = defineComponent({
  props: ['widgetId', 'config'],
  setup(props) {
    const stats = ref({ value: 42, label: 'Показатель' });
    
    return () => h('div', 
    { 
        class: 'stats-widget' 
    }, 
    [
      h('div', { class: 'stat-value' }, stats.value.value),
      h('div', { class: 'stat-label' }, stats.value.label)
    ]
    );
  }
});
const WidgetStatistic = defineComponent({
  props: ['widgetId', 'config'],
  async setup(props) {
    const stats = ref({label: 'Статистика по пользователям' });
    const employees = ref<Employees | undefined>()
    employees.value = await http_sevice.employees_service.get_employees_with_status();
    const interval = setInterval(async () => 
    {
      employees.value = await http_sevice.employees_service.get_employees_with_status();
    }, 1000 * 60 * 10);

    onUnmounted(() => 
    {
      clearInterval(interval);
    });

    if(employees.value)
    return () => h(SimpleStatistic, 
    { 
        employees: employees.value as Employees,
        class: 'statistics-widget' 
    });
  }
});

const WidgetNotifications = defineComponent({
  props: ['widgetId', 'config'],
  async setup(props) {
    const stats = ref({label: 'Напоминания' });

    return () => h(NotifyViewer, 
    { 
        items: time_warnings.value,
        class: 'notifications-widget' 
    });
  }
});

const WidgetAllStatistic = defineComponent({
  props: ['widgetId', 'config'],
  async setup(props) {
    const stats = ref({label: 'Статистика по всем параметрам' });
    const employees = ref<Employees | undefined>()
    employees.value = await http_sevice.employees_service.get_employees_with_status();
    const interval = setInterval(async () => 
    {
      employees.value = await http_sevice.employees_service.get_employees_with_status();
    }, 1000 * 60 * 10);

    onUnmounted(() => 
    {
      clearInterval(interval);
    });

    if(employees.value)
    return () => h(EmployeesStatistic, 
    { 
        employees: employees.value,
        class: 'statistics-widget' 
    });
  }
});
</script>

<script lang="ts" setup>
const emitter = inject<Emitter<Events>>('emitter') as Emitter<Events>;

// Состояние дашборда
const isEditMode = ref(false);
const isDarkMode = ref(false);
const showAddWidgetModal = ref(false);
const gridEl = ref<HTMLElement>();

// Доступные виджеты
const availableWidgets = {
  clock: {
    component: WidgetClock,
    name: 'Часы',
    description: 'Отображает текущее время',
    icon: '⏰',
    defaultSize: { w: 2, h: 2 }
  },
  statistic: {
    component: WidgetStatistic,
    name: 'Статистика',
    description: 'Показывает статистику по сотрудникам',
    icon: '📊',
    defaultSize: { w: 10, h: 20 }
  },
  employees_statistic: {
    component: WidgetAllStatistic,
    name: 'Общая статистика',
    description: 'Показывает всю статистику',
    icon: '📊',
    defaultSize: { w: 10, h: 20 }
  },
  notifications: {
    component: WidgetNotifications,
    name: 'Напоминания',
    description: 'Показывает ваши напоминания',
    icon: '🔔',
    defaultSize: { w: 4, h: 6 }
  },
};

// Макет виджетов
interface WidgetLayout {
  i: string;
  x: number;
  y: number;
  w: number;
  h: number;
}

const widgetsLayout = ref<WidgetLayout[]>([]);
const widgetConfigs = ref<Record<string, any>>({});

// Загрузка из localStorage
const loadDashboard = () => 
{
  const data = load_from_localstorage<{
    layout: WidgetLayout[],
    configs: Record<string, any>
  }>('dashboard-layout');
  if (data) 
  {
    //const data = JSON.parse(saved);
    widgetsLayout.value = data.layout || [];
    widgetConfigs.value = data.configs || {};
  }
};

// Сохранение в localStorage
const saveDashboard = () => {
  const data = {
    layout: widgetsLayout.value,
    configs: widgetConfigs.value
  };
  save_to_localstorage('dashboard-layout', data);
};

// Управление виджетами
const addWidget = (type: string) => 
{
  const widgetId = `widget_${type}`;
  const widgetDef = availableWidgets[type as keyof typeof availableWidgets];

  // match(type)
  // .with('statistic', async () => {
  //   widgetConfigs.value[widgetId] = {
  //   type: type,
  //   employees: await http_sevice.employees_service.get_employees_with_status(),
  //   createdAt: new Date().toISOString()
  // };
  // })
  // .otherwise(()=> widgetConfigs.value[widgetId] = {
  //   type: type,
  //   createdAt: new Date().toISOString()
  // })

  widgetConfigs.value[widgetId] = {
    type: type,
    createdAt: new Date().toISOString()
  }

  widgetsLayout.value.push({
    i: widgetId,
    x: 0,
    y: 0,
    w: widgetDef.defaultSize.w,
    h: widgetDef.defaultSize.h
  });
  
  
  
  showAddWidgetModal.value = false;
  saveDashboard();
};

const removeWidget = (widgetId: string) => {
  widgetsLayout.value = widgetsLayout.value.filter(w => w.i !== widgetId);
  delete widgetConfigs.value[widgetId];
  saveDashboard();
};

const getWidgetComponent = (widgetId: string) => {
  const config = widgetConfigs.value[widgetId];
  return availableWidgets[config.type as keyof typeof availableWidgets]?.component;
};

const getWidgetTitle = (widgetId: string) => {
  const config = widgetConfigs.value[widgetId];
  return availableWidgets[config.type as keyof typeof availableWidgets]?.name || 'Виджет';
};

const getWidgetConfig = (widgetId: string) => {
  return widgetConfigs.value[widgetId] || {};
};

// Обработчики событий
const toggleEditMode = () => {
  isEditMode.value = !isEditMode.value;
  if (!isEditMode.value) {
    saveDashboard();
  }
};

const toggleDarkMode = () => {
  isDarkMode.value = !isDarkMode.value;
};

const onLayoutUpdated = (newLayout: WidgetLayout[]) => {
  widgetsLayout.value = newLayout;
};

const onWidgetDoubleClick = (widget: WidgetLayout) => {
  if (isEditMode.value) {
    // Редактирование виджета
    //notify_service.(`Редактирование виджета: ${widget.i}`);
  }
};

// Жизненный цикл
onMounted(() => {
  loadDashboard();
  //emitter.on('widget-added', (type: string) => addWidget(type));
});

onBeforeUnmount(() => {
  //emitter.off('widget-added');
});

// Следим за изменениями макета
watch(widgetsLayout, saveDashboard, { deep: true });
</script>

<style lang="scss" scoped>
.n-dashboard {
  padding: 20px;
  min-height: 100vh;
  background: var(--n-color);
}

.dashboard-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding: 0 10px;
  
  h2 {
    margin: 0;
    font-size: 24px;
  }
  
  .header-controls {
    display: flex;
    gap: 10px;
  }
}

.dashboard-grid {
  position: relative;
  min-height: 500px;
}

.widget {
  background: var(--n-color);
  border-color: var(--n-close-icon-color);
  border: 1px solid var(--n-close-icon-color);
  border-radius: 8px;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  
  &-editing {
    border: 2px dashed #1890ff;
  }
  
  &-header {
    // padding: 12px;
    // border-bottom: 1px solid #f0f0f0;
    // display: flex;
    // justify-content: space-between;
    // align-items: center;
    // background: transparent;
  }
  
  &-title {
    font-weight: 600;
    font-size: 14px;
  }
  
  &-controls {
    display: flex;
    gap: 5px;
  }
  
  &-content {
    flex: 1;
    padding: 12px;
    overflow: auto;
  }
}

.add-widget-btn {
  position: fixed;
  top: 25px;
  right: 30px;
  z-index: 1000;
}

.add-widget-modal {
  .widgets-list {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 15px;
    margin-top: 20px;
  }
  
  .widget-item {
    padding: 20px;
    border: 2px solid #e0e0e0;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.3s;
    text-align: center;
    
    &:hover {
      border-color: #1890ff;
      transform: translateY(-2px);
    }
  }
  
  .widget-icon {
    font-size: 24px;
    margin-bottom: 10px;
  }
  
  .widget-name {
    font-weight: 600;
    margin-bottom: 5px;
  }
  
  .widget-desc {
    font-size: 12px;
    color: #666;
  }
}

// Стили для конкретных виджетов
.clock-widget {
  text-align: center;
  
  .time {
    font-size: 24px;
    font-weight: bold;
    color: #1890ff;
  }
}

.stats-widget {
  text-align: center;
  
  .stat-value {
    font-size: 32px;
    font-weight: bold;
    color: #52c41a;
  }
  
  .stat-label {
    font-size: 14px;
    color: #666;
  }
}
</style>