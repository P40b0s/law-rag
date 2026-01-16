<template lang="pug">
n-card.fios-style
  div(v-if="sorted_fios" v-for="fio in sorted_fios")
    .fio-content 
      .fio {{fio.surname}} {{fio.first_name[0]}}.{{fio.second_name[0]}}.
      tag-with-progress.tag(
        :color="get_status(fio.status_id).color"
        :logo="get_status(fio.status_id).logo"
        :status_name="fio.status")
    
</template>

<script lang="ts" setup>
import {
  NDescriptions,
  NDescriptionsItem,
  NSpace,
  NLayout,
  NLayoutSider,
  NLayoutHeader,
  NLayoutContent,
  NDivider,
  NCard,
  NTag
} from 'naive-ui'
import { type Fio, type DepartmentsStatistic} from '@/types/statistic';
import { computed } from 'vue'
import TagWithProgress from './TagWithProgress.vue';
import { useDictionaries } from '@/composables/useDictionaries';
const props = defineProps<{
  fios?: Fio[]
}>()
const {get_status} = useDictionaries();
const sorted_fios = computed(() => props.fios?.sort((a,b) => a.surname > b.surname ? 1 : 0))
</script>

<style lang="scss" scoped>
.fio-content
{
  width: 100%;
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  &:hover
  {
    background-color: #918d8d50;
  }
}
.fio
{
  font-size: 20px;
  flex-basis: 70%;
}
.tag
{
  flex-basis: 30%;
}
.fios-style
{
  display: flex;
  flex-direction: column;
}
// Адаптивность
@media (max-width: 1200px) 
{
  .row 
  {
    grid-template-columns: 150px repeat(auto-fit, minmax(250px, 1fr));
  }
}

@media (max-width: 768px) 
{
  .grid-table 
  {
    font-size: 14px;
  }
  
  .row 
  {
    grid-template-columns: 120px repeat(auto-fit, minmax(200px, 1fr));
  }
  
  .data-cell 
  {
    grid-template-columns: 1fr;
  }
  
  .department-subcell,
  .count-subcell,
  .employees-subcell 
  {
    padding: 8px 4px;
  }
}
</style>