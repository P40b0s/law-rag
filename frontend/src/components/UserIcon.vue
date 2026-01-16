<template lang="pug">
div.avatar-container
    div.text-up(v-if="user && name_visible") {{ user.first_name }}
    span.text-up(v-else-if="props.show_name")
    div.avatar-wrapper(@mouseover="mouse_over" @mouseleave="mouse_leave")
        n-avatar(v-if="avatar" :src="avatar" round lazy :size="props.size")
        n-avatar(v-else :src="homer_ico" round lazy :size="props.size")
    span.text-bottom(v-if="user && name_visible") {{ user.surname }}
    span.text-bottom(v-else-if="props.show_name")
</template>
        
<script lang="ts">
import {UserAvatarFilledAlt} from '@vicons/carbon'
import { defineComponent, ref, onMounted, watch, computed} from 'vue'
import {NTooltip, NAvatar, NIcon} from 'naive-ui'
import { type Document } from "@/types/document"
import { Error } from '@vicons/carbon'
import { base64_to_uint8_array } from '@/services/helpers'
import { useImage } from '@/composables/useImage'
import { type UserInfo } from '@/types/user_info'
import { homer_ico } from '@/services/svg'
import { http_sevice } from '@/services/http_service/http_service'
</script>

<script lang="ts" setup>
//n-icon(v-if="user == undefined && props.visible_if_not_found" size="35" :component="Error" color='#d51a1a')
interface Props 
{
    user_id: string,
    show_name?: boolean,
    visible_if_not_found?: boolean
    size?: number
}
const name_visible = ref(false);
const props = withDefaults(defineProps<Props>(),
{
    visible_if_not_found: false,
    size: 35
})
const mouse_over = async () =>
{
    if(props.show_name)
    {
        if (!user.value)
        user.value = await http_sevice.get_user(props.user_id)
        name_visible.value = true;
    }
}
const mouse_leave = async () =>
{
    name_visible.value = false;
    
}
const user = ref<UserInfo>();
const {get_avatar, get_avatars} = useImage();
onMounted(async ()=> get_avatar(props.user_id));
const avatar = computed(()=> get_avatars().value.get(props.user_id));


// onMounted(async ()=>
// {
//     avatar.value = await get_avatar(props.user_id);
// })

// const user = ref(users.value.find(f=>f.id == props.user_id));
// //const avatar = ref<string|undefined>(avatars.value.get(props.user_id ?? -1));
// watch(users, (n) =>
// {
//     user.value = users.value.find(f=>f.id == props.user_id);
//     avatar.value = avatars.value.get(props.user_id ?? -1);
// })
</script>
    
<style lang="scss" scoped>
.mr-4
{
    margin-right: 4px;
}
.text-up 
{
    margin-bottom: 9px;
    position: relative;
    height: 10px;
}
.text-bottom 
{
    margin-top: -15px;
    position: relative;
    height: 10px;
}
.avatar-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  background-color: transparent;
}

.avatar-wrapper {
  position: relative;
  border-radius: 50%;
  padding: 2px; /* Отступ для свечения */
  z-index: 1;
  opacity: 1;
  
  &::before {
    content: '';
    position: absolute;
    top: 3px;
    left: 3px;
    right: 3px;
    bottom: 10px;
    border-radius: 50%;
    background: transparent;
    z-index: -1;
    transition: all 0.3s ease;
  }

  &:hover::before {
    box-shadow: 
      0 0 10px 3px rgba(100, 200, 255, 0.7), /* Голубое свечение */
      0 0 20px 5px rgba(100, 200, 255, 0.4); /* Рассеянное свечение */
  }
}

/* Анимация пульсации (опционально) */
@keyframes pulse {
  0% { box-shadow: 0 0 5px 2px rgba(100, 200, 255, 0.5); }
  50% { box-shadow: 0 0 15px 5px rgba(100, 200, 255, 0.8); }
  100% { box-shadow: 0 0 5px 2px rgba(100, 200, 255, 0.5); }
}

.avatar-wrapper.active::before {
  animation: pulse 2s infinite;
}
// .avatar-wrapper::before {
//   background: linear-gradient(45deg, #00ffff, #ff00ff);
//   opacity: 0;
//   transition: opacity 0.3s;
// }

// .avatar-wrapper:hover::before {
//   opacity: 0.7;
//   box-shadow: 
//     0 0 10px 5px #00ffff,
//     0 0 20px 10px #ff00ff;
// }
</style>