<template lang="pug">
div.users-panel
	n-table.tbl(:bordered="false" :single-line="false")
		thead
			tr
				th.id-header Имя пользователя
				th.fio-header ФИО
				th.role-header Роль
				th.avatar-header Аватар
				th.buttons-header
					.actions-header
						div Действия
						n-tooltip(v-if="element_is_visible") Создать нового пользователя
							template(#trigger)
								n-button( quaternary circle @click="create_new_handler")
									template(#icon)
										svg-icon(:svg="add_user_ico" :size="34")
											
		tbody.scrollable-tbody
			template(v-for="u in users")
				tr
					td.id-body {{u.username}}
					td.fio-body {{u.surname + " " + u.first_name + " " + u.second_name}}
					td.role-body {{get_role_name(u.role)}}
					td.avatar-body
						.avatar
							UserIcon(:user_id="u.id" :size="60")
					td.buttons-body
						.buttons
							n-button(v-if="element_is_visible" @click="edit_handler(u.id)") Редактировать
							n-button(:disabled="delete_process_id == u.id || u.id == current_user.id" v-if="element_is_visible" @click="delete_handler(u.id)") Удалить
					
	.profile
		profile-editor(v-if="profile" v-model:profile="profile" v-model:new_user="is_new")
		//- n-button.new-button(@click="create_new_handler") Создать нового пользователя
</template>
    
<script lang="ts">
import { ref, type Component, watch, inject, onMounted, onUnmounted, computed, onBeforeUnmount, h, toRefs } from 'vue';
import { type Events, type Emitter } from '../services/emitter';
import { NForm, NFormItem, NInput, NButton, NIcon, NTooltip, darkTheme, NCard, NTable, NScrollbar} from 'naive-ui';
import { notify_service } from '@/services/notification_service';
import useUser from '@/composables/useUser';
import {Upload} from '@vicons/carbon'
import add_user_ico from '@svg/add_user.svg';
//import  user_service  from '../services/user_service';
import UserIcon from '@components/UserIcon.vue';
import { match } from 'ts-pattern';
import { type Role, roles } from '@/types/user_role';
import {type UserInfo} from '@/types/user_info';
import ProfileEditor from '@/components/ProfileEditor.vue'
import { http_sevice } from '@/services/http_service/http_service';
import Loader from '@/components/Loader.vue';
import SvgIcon from '@/components/SvgIcon.vue';
import useVisible from '@/composables/useVisible';
</script>

<script lang="ts" setup>
const emitter = inject<Emitter<Events>>('emitter') as Emitter<Events>;
const profile = ref<UserInfo|null>(null);
const is_new = ref<boolean>(false);
const {visible} = useVisible()
const {get_role_name, get_user} = useUser();
const element_is_visible = visible(['Administrator', 'User'], ['All'])
const users = ref<UserInfo[]>([]);
const last_edit_user_id = ref<string|null>(null);
const delete_process_id = ref("");
const current_user = get_user();
const update_profile_event = async () =>
{
	users.value = await http_sevice.get_users();
	if(last_edit_user_id.value)
	{
		console.log(last_edit_user_id.value)
		const user = await http_sevice.get_user(last_edit_user_id.value);
		if(user)
		{
			profile.value = user;
		}
		else
		{
			profile.value = null;
		}
	}
	else profile.value = null;
}
const add_user = () =>
{

}
onMounted(async () => 
{
	emitter.on('update_profile', update_profile_event)
	users.value = await http_sevice.get_users();
});
onUnmounted(() => 
{
	emitter.off('update_profile', update_profile_event)
})

const edit_handler = async (user_id: string) =>
{
	is_new.value = false;
	const user = await http_sevice.get_user(user_id);
	if(user)
	{
		profile.value = user;
		last_edit_user_id.value = user_id;
	}
}

const delete_handler = async (user_id: string) =>
{
	delete_process_id.value = user_id;
	const user = await http_sevice.delete_user(user_id);
}


const create_new_handler = async () =>
{
	is_new.value = true;
	const user = {
		first_name: "",
		username: "",
		second_name: "",
		surname: "",
		id: "",
		role: 'User',
		password: "",
		permissions: ["View"]
	} as UserInfo
	profile.value = user;
}

</script>
    
<style lang="scss" scoped>
$id-width: 80px;
$fio-width: 100px;
$role-width: 85px;
$avatar-width: 45px;
$buttons-width: 90px;
.users-panel
{
	height: calc(100vh - 20px);
	display: flex;
	flex-direction: row;
    flex: 1;
	gap: 10px;
	width: calc(100vw - 100px);
    overflow: hidden;
    overflow-y: hidden;
    overflow-x: hidden;
    box-sizing: border-box;
    margin-left: 10px;
    margin-top: 5px;
}
.tbl
{
	flex: 1.3;
  	display: flex;
  	flex-direction: column;
}
.profile
{
	flex: 1;
  	display: flex;
  	flex-direction: column;
	align-items: start;
}
thead
{
	display: table;
  	width: 100%;
  	table-layout: fixed;
}
tbody tr 
{
  display: table;
  width: 100%;
  table-layout: fixed;
}

.fio-header
{
	width: $fio-width;
	height: 50px;
	text-align: center;
}
.id-header
{
	width: $id-width;
	height: 50px;
	text-align: center;
}
.role-header
{
	width: $role-width;
	text-align: center;
}
.avatar-header
{
	width: $avatar-width;
	text-align: center;
}
.buttons-header
{
	width: $buttons-width;
	text-align: center;
}
.actions-header
{
	display: flex;
	flex-direction: row;
	gap: 4px;
	align-items: center;
	justify-content: center;
}
.id-body
{
	width: $id-width;
	height: 50px;
	text-align: center;
}
.fio-body
{
	width: $fio-width;
	height: 50px;
	text-align: center;
}
.role-body
{
	width: $role-width;
	text-align: center;
}
.avatar-body
{
	width: $avatar-width;
	text-align: center;
}
.buttons-body
{
	width: $buttons-width;
	text-align: center;
}
.avatar
{
	display: flex;
	flex-direction: column;
	align-items: center;
	gap: 5px;

}
.buttons
{
	display: flex;
	flex-direction: column;
	gap: 5px;
}

.trow
{
	height: 50px;
	vertical-align: middle;
}
.scrollable-tbody
{
	display: block;
  	overflow-y: auto;
  	max-height: calc(100vh - 50px); /* Подстройте под ваш интерфейс */
}
.new-button
{
	align-self: stretch;
}
</style>