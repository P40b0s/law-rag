<template lang="pug">
.progress-container(v-if="props.date_from && props.date_to")
  n-tag(
    size="medium" 
    :color="{borderColor: props.color, textColor: props.color}"
  ) 
    .tag-content
      template(v-if="props.logo")
        svg-icon-native(
          :size="18" 
          :svg="props.logo" 
        )
      .status-text {{props.status_name}} {{formatDate(props.date_from) }} - {{formatDate(props.date_to) }}
  n-progress.pg(
    :percentage="percentage" 
    :show-indicator="false" 
    status="info" 
    :height="27"
    :border-radius="2"
    :fill-border-radius="0"
  )

.progress-container(v-else)
  n-tag(
    size="medium" 
    :color="{borderColor: props.color, textColor: props.color}"
  )
    .tag-content
      template(v-if="props.logo")
        svg-icon-native(
          :size="18" 
          :svg="props.logo" 
        )
      .status-text {{props.status_name}}
  n-progress.pg(
    v-if="props.percentage" 
    :percentage="props.percentage" 
    :show-indicator="false" 
    status="info" 
    :height="27"
    :border-radius="2"
    :fill-border-radius="0"
  )
</template>
  
<script lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import {
  NTag,
  NProgress,
} from 'naive-ui'
import { DateFormat, DateTime } from '@/services/date';
import SvgIconNative from './SvgIconNative.vue';
</script>
<script lang="ts" setup>
const props = defineProps<{
  date_from?: DateTime,
  date_to?: DateTime,
  status_name: string,
  logo?: string,
  color: string,
  percentage?: number
}>()
const percentage = ref(0);
const days_count = (start_date: DateTime| null, end_date: DateTime| null) =>
{
  if(start_date && end_date)
  {
    return start_date.days_between(end_date)
  }
  else return 0
}
const formatDate = (date: DateTime| null): string| null => 
{
  return date ? date.to_string(DateFormat.CalendarFormat) : ""
}
watch(() => props, (new_props, old_props) =>
{
  const all_days = days_count(new_props.date_from ?? null, new_props.date_to ?? null); //100%
  const days_left = days_count(DateTime.new(), new_props.date_to ?? null);
  if(days_left > 0)
      percentage.value = 100 - ((days_left / all_days) * 100)
}, 
{immediate: true, deep: true})

</script>
  
<style scoped lang="scss">
.progress-container {
  position: relative;
  display: inline-block; /* или block, в зависимости от нужной ширины */
}

.pg {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  opacity: 0.15;
  z-index: 1;
  pointer-events: none; /* чтобы не блокировал клики по тегу */
}

.tag-content {
  position: relative;
  z-index: 2; /* чтобы контент был поверх прогресса */
  display: flex;
  align-items: center;
  gap: 5px;
}

.status-text {
  position: relative;
}
</style>