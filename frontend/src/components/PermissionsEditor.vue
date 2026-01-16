<template lang="pug">
n-scrollbar(style="max-height: 130px")
    .priv
        n-checkbox(v-for="p in all_permissions" :disabled="props.disabled" @update:checked="checked" v-model:checked="p.checked" :key="p.name") {{p.description}}
</template>
        
<script lang="ts">
import {UserAvatarFilledAlt} from '@vicons/carbon'
import { defineComponent, ref, onMounted, watch, computed} from 'vue'
import {NTooltip, NAvatar, NIcon, NCheckbox, NScrollbar} from 'naive-ui'
import { type Document } from "@/types/document"
import useUser from '@composables/useUser'
import { Error } from '@vicons/carbon'
import { base64_to_uint8_array, sleepNow } from '@/services/helpers'
import { type Permission, permissions } from '@/types/permission'
import { match } from 'ts-pattern'
</script>

<script lang="ts" setup>
interface Props 
{
    value?: Permission[],
    disabled: boolean
}
interface Checked
{
    name: Permission,
    description: string,
    checked: boolean
}
const props = defineProps<Props>();
const emits = defineEmits<{
    (e: 'update:value', profile: Permission[]): void
}>()
const all_permissions = ref<Checked[]>([]);
const {get_permission_description} = useUser();
watch(() => props.value, (n) =>
{
    all_permissions.value = [];
    permissions.forEach(f=>
    {
        let checked = n?.includes(f);
        const descriprion = get_permission_description(f);
        all_permissions.value.push({name: f, description: descriprion, checked: checked ?? false})
    })
}, {immediate: true})
const checked = async (v: string) => 
{
    await sleepNow(200); //hmm событие вызывается до того как сработает привязка к 
    // all_privileges это конечно решает вопрос но похоже на костыль
    const priv = all_permissions.value.filter(f=>f.checked == true).map(m=>m.name);
    emits('update:value', priv);
}
// const sorted_privileges = computed(() => 
// {
//   return [...all_privileges.value].sort((a, b) => 
//     Number(b.checked) - Number(a.checked))
// });
</script>
    
<style lang="scss" scoped>
.mr-4
{
    margin-right: 4px;
}
.priv
{
    display: flex;
    flex-direction: column;
}
</style>