<template lang="pug">
n-select(:disabled="props.disabled" @update:value="handle_update_role" :options="options" v-model:value="selected_item")
</template>
        
<script lang="ts">
import {UserAvatarFilledAlt} from '@vicons/carbon'
import { defineComponent, ref, onMounted, watch, computed} from 'vue'
import {NTooltip, NAvatar, NIcon, NCheckbox, NSelect, type SelectOption} from 'naive-ui'
import { type Document } from "@/types/document"
import useUser from '@composables/useUser'
import { Error } from '@vicons/carbon'
import { base64_to_uint8_array } from '@/services/helpers'
import { type Permission, permissions } from '@/types/permission'
import { match } from 'ts-pattern'
import { roles, type Role } from '@/types/user_role'
</script>

<script lang="ts" setup>
interface Props 
{
    value: Role,
    disabled: boolean
}
interface SelectItemInterface
{
    label: string,
    value: Role
}
const props = defineProps<Props>();
const emits = defineEmits<{
    (e: 'update:value', role: Role): void
}>()
const {get_role, get_role_name} = useUser();
const current_role = get_role();
const selected_item = ref<string|null>(null);
const options: SelectItemInterface[] = roles.map(r=>
{
    return {
          label: get_role_name(r),
          value: r,
        }
})
const handle_update_role = (value: Role, option: SelectOption) => 
{
    emits('update:value', value)
}
watch(() => props.value, () =>
{
    selected_item.value = props.value;
}, {immediate: true})
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