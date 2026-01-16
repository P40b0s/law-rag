<template lang="pug">
profile-editor(v-model:profile="profile" theme_selector, exit)
</template>
    
<script lang="ts">
import { ref, type Component, watch, inject, onMounted, onUnmounted, computed, onBeforeUnmount, h, toRefs, readonly } from 'vue';
import { type Events, type Emitter } from '../services/emitter';
import { NForm, NFormItem, NInput, NButton, darkTheme, NDivider, NTooltip, NCard, type FormInst, type FormRules, type FormItemRule, type FormValidationError } from 'naive-ui';
import { notify_service } from '@/services/notification_service';
import { http_sevice } from '@/services/http_service/http_service';
import { type UserInfoUpdate, type UserInfo } from '@/types/user_info';
import Loader from '@components/Loader.vue'
import { compressImage } from '@/services/helpers';
import ProfileEditor from '@/components/ProfileEditor.vue'
//import  user_service  from '../services/user_service';

</script>
<script lang="ts" setup>
const emitter = inject<Emitter<Events>>('emitter') as Emitter<Events>;

const profile = ref<UserInfo>();
const update_profile_emit = async () =>
{
    const pr = await http_sevice.get_profile();
    if(pr)
    {
        profile.value = pr;
    }
}
onMounted(async ()=> 
{
    emitter.on('update_profile', update_profile_emit);
    const pr = await http_sevice.get_profile();
    if(pr)
    {
        profile.value = pr;
    }
})
onUnmounted(() => emitter.off('update_profile', update_profile_emit))

</script>
    
<style lang="scss" scoped>
</style>