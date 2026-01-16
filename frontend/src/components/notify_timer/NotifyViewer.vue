<template lang="pug">
div(style="gap: 2px;" )
  div(v-for="tw in items" :key="tw.id" v-if="items")
    .notification-item
      n-thing
        template(#avatar)
          n-tooltip
            template(#trigger)
              n-progress.progress(
                v-if="tw.minutes_left !== undefined && tw.minutes_left !== 0"
                style="width: 70px"
                type="circle"
                :stroke-width="10"
                status="info"
                :percentage="tw.progress"
                color="rgba(255, 43, 15, 0.8)"
                rail-color="rgba(84, 237, 33, 0.8)"
              )
                template(#default)
                  .time {{ dateTimeString(tw)  }}
              .time(v-else) {{ dateTimeString(tw)  }}
            span
              | {{ tw.hours === 0
              |   ? `Осталось ${tw.minutes_left} мин.`
              |   : `Осталось ${tw.hours} час. ${tw.minutes} мин.` }}
        template(#header)
          div {{ tw.text }}
        template(#description)
          div(v-if="tw.weekDay") {{ tw.text }}
      n-divider
</template>

<script setup lang="ts">
import { ref, watch, toRef, type PropType, h, type Ref, onMounted } from 'vue'
import { NButton, NIcon, NThing, NDivider, NProgress, NTooltip, useNotification, type NotificationType } from 'naive-ui'
import { WarningOutline } from '@vicons/ionicons5'
import { type TimeWarning, useTime } from './use_time_warnings'
import { type TimeProgress, dateToString, timeLeft } from '../../services/date'
const props = defineProps({
  items: {
    type: Array as PropType<TimeWarning[]>,
    required: true
  },
  notifyTimeout: {
    type: Number,
    default: 30
  },
  notifyShowDelay: {
    type: Number,
    default: 5
  }
})

const items = toRef(props, 'items') as Ref<(TimeWarning & TimeProgress)[]>
console.log('NotifyViewer items', items.value);
onMounted(() => 
{
  //items.value = toRef(props, 'items').value as Ref<(TimeWarning & TimeProgress)[]> //props.items.map(i => ({ ...i }))
})
//const items = toRef(props, 'items') as Ref<(TimeWarning & TimeProgress)[]>;
const emit = defineEmits<{
  (e: 'update:items', values: TimeWarning[]): void
}>()

const { timer } = useTime()
const notification = useNotification()
const current_date = ref(new Date())
const getDayName = (day: number): string => 
{
  const dayNames = ['Вс', 'Пн', 'Вт', 'Ср', 'Чт', 'Пт', 'Сб']
  return dayNames[day]
}
function refreshTimeProgress(t: TimeWarning & TimeProgress) 
{
  const tl = timeLeft(t.warningTime)
  if (tl) {
    t.progress = tl.progress
    t.minutes_left = tl.minutes_left
    t.minutes = tl.minutes
    t.hours = tl.hours
  }
}

function notify(type: NotificationType, tw: TimeWarning & TimeProgress) {
  const n = notification.create({
    type,
    title: `Осталось ${tw.minutes_left} мин.`,
    description: `Автоматическое напоминание на ${tw.time}`,
    content: tw.text,
    action: () =>
      h(
        NButton,
        {
          text: true,
          type: 'primary',
          onClick: () => {
            tw.showNotify = false
            n.destroy()
          }
        },
        { default: () => 'Больше не показывать' }
      ),
    avatar: () =>
      h(
        NProgress,
        {
          style: { width: '100px', marginBottom: '20px' },
          type: 'circle',
          status: 'info',
          indicatorPosition: 'inside',
          percentage: tw.progress,
          color: 'rgba(255, 43, 15, 0.8)',
          railColor: 'rgba(84, 237, 33, 0.8)'
        },
        {
          default: () =>
            h(NIcon, {
              style: { marginTop: '14px' },
              size: '20',
              color: 'rgba(255, 43, 15, 0.8)',
              component: WarningOutline
            })
        }
      ),
    duration: 25500,
    keepAliveOnHover: true
  })
}

function dateTimeString(tw: TimeWarning) {
  let ts = ''
  if (tw.date) {
    if (tw.time) ts = ts + tw.time
    else ts = ts + dateToString(tw.date)
  } else if (tw.time) ts = ts + tw.time
  return ts
}

function check_notify(t: TimeWarning & TimeProgress) {
  if (t.startNotifyTime && current_date.value.getMinutes() % props.notifyShowDelay === 0) {
    const c_time = new Date().setSeconds(0, 0)
    if (c_time >= t.startNotifyTime && t.showNotify) {
      notify('warning', t)
    }
  }
}

function set_visibility(tw: TimeWarning & TimeProgress) {
  const c_time = new Date().setSeconds(0, 0)
  if ((tw.warningTime as number) < c_time) tw.isVisible = false
  else {
    tw.isVisible = true
    check_notify(tw)
  }
}

function checkTimeWarnings() {
  items.value.forEach(t => {
    if (t.time) {
      refreshTimeProgress(t)
      if (t.date) {
        if (t.date.getTime() === timer.value.date_without_time) {
          set_visibility(t)
        }
      } else if (t.weekDay) {
        if (t.weekDay.some(s => s === timer.value.week_day)) {
          set_visibility(t)
        } else {
          t.isVisible = false
        }
      } else {
        set_visibility(t)
      }
    } else {
      if (t.weekDay && t.weekDay.some(s => s === timer.value.week_day)) {
        t.isVisible = true
      } else if (t.date && t.date.getTime() === timer.value.date_without_time) {
        t.isVisible = true
      }
    }
  })
}

watch(
  () => timer.value,
  () => {
    if (timer.value.is_midnight) {
      props.items.forEach(f => (f.showNotify = true))
    }
    checkTimeWarnings()
  }
)

watch(
  () => props.items,
  () => {
    checkTimeWarnings()
  }
)

checkTimeWarnings()
</script>
<style lang="scss" scoped>
.notification-item
{
  display: flex;
  flex-direction: column;
  align-items: start;
  font-size: 18px;
  gap: 10px;
}
.progress
{
  flex-basis: 150px;
}
.time
{
  font-weight: 700;
  color: green;
  flex-basis: 100px;
}
</style>