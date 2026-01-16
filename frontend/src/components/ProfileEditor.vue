<template lang="pug">
.profile(v-if="profile != null")
	n-radio-group.theme-buttons(v-if="props.theme_selector" v-model:value="theme" @update:value="change_theme" name="theme_group")
		n-radio-button(value="dark" label="Темная тема")
		n-radio-button(value="light" label="Светлая тема")
	.main
		n-card.left
			n-form(
				ref="formRef"
				:label-width="80"
				:model="profile"
				:rules="rules"
				size="medium")
					n-form-item(v-if="role == 'Administrator' && !props.new_user" label="Идентификатор" path="id")
						n-input(v-model:value="profile.id" disabled)
					n-form-item(label="Фамилия" path="surname")
						n-input(v-model:value="profile.surname" placeholder="Введите фамилию")
					n-form-item(label="Имя" path="first_name")
						n-input(v-model:value="profile.first_name" placeholder="Введите имя")
					n-form-item(label="Отчество" path="second_name")
						n-input(v-model:value="profile.second_name" placeholder="Введите отчество")
					n-form-item(label="Имя пользователя" path="username")
						n-input(v-model:value="profile.username" :disabled="role != 'Administrator' || !props.new_user" placeholder="")
					n-form-item(label="Пароль" path="password")
						n-input(v-model:value="profile.password" :placeholder="props.new_user ? 'введите пароль' : '(не менять)'")
					n-form-item(label="Привязка сотрудника" path="assotiate")
						n-select(v-model:value="selected_assotiated_employee" :options="employees_list" placeholder="Привязать сотрудника")
		n-card.right
			n-form-item.avatar-form(label="Аватар" path="avatar")
				label
					input(
					type="file" 
					accept="image/*" 
					style="display: none"
					@change="handleFileUpload")
					n-tooltip(v-if="previewUrl") Нажмите для загрузки изображения
						template(#trigger)
							img.ava-img(:src="previewUrl")
					img.ava-img(:src="HomerIcon" v-else)
			n-form-item(label="Роль" path="role")
				role-selector(v-model:value="profile.role" :disabled="element_is_disabled")
			n-form-item(label="Права" path="permissions")
				permissions-editor(v-model:value="profile.permissions" :disabled="element_is_disabled")
	n-button.save-button(:disabled="not_valid" @click="save" :type="not_valid ? 'error' : 'success'" @mouseover="mouse_over") Сохранить
	n-button.save-button(v-if="props.exit" @click="exit" type="error") Выйти из сессии
loader.ld(v-else status="Загрузка профиля...")
</template>
    
<script lang="ts">
import { ref, type Component, watch, inject, onMounted, onUnmounted, computed, onBeforeUnmount, h, toRefs, readonly } from 'vue';
import { type Events, type Emitter } from '../services/emitter';
import { NForm, NFormItem, NSelect, NInput, NRadioButton, NInputNumber, NRadioGroup, NButton, darkTheme, NDivider, NTooltip, NCard, type FormInst, type FormRules, type FormItemRule, type FormValidationError } from 'naive-ui';
import { notify_service } from '@/services/notification_service';
import { http_sevice } from '@/services/http_service/http_service';
import { type UserInfoUpdate, type UserInfo, type CreateUserPayload, type AdminUserInfoUpdate } from '@/types/user_info';
import Loader from '@components/Loader.vue'
import { compressImage } from '@/services/helpers';
import PermissionsEditor from './PermissionsEditor.vue';
import RoleSelector from './RoleSelector.vue';
import useUser from '@/composables/useUser';
import HomerIcon from '@svg/homer.svg'
import { roles } from '@/types/user_role';
import { type Theme, useTheme } from '@/composables/useTheme';
import { boolean } from 'zod';
import { useImage } from '@/composables/useImage';
import useVisible from '@/composables/useVisible';
import { permissions } from '@/types/permission';
import { type AssotiatedEmployee } from '@/types/employees';
interface Props
{
	profile?: UserInfo,
	theme_selector?: boolean,
	exit?: boolean,
	new_user?: boolean
}
</script>
<script lang="ts" setup>
const emitter = inject<Emitter<Events>>('emitter') as Emitter<Events>;

const props = withDefaults(defineProps<Props>(), 
{
  new_user:  false,
})

const {visible, disabled} = useVisible();
const element_is_visible = visible(['Administrator', 'User']);
const element_is_disabled = disabled(['User']);
const is_all_permissions = visible(['Administrator'], ['All']);
const emits = defineEmits<{
    (e: 'save', profile: UserInfoUpdate): void,
    (e: 'update:profile', profile: UserInfo): void,
	(e: 'update:new_user', b: boolean): void,
}>()
const formRef = ref<FormInst | null>(null)
const profile = ref<UserInfo & {password: string}|null>(null);
const assotiated_employee = ref<AssotiatedEmployee|null>(null);
const selected_assotiated_employee = ref<string|null>(null);
const employees_list = ref<{label: string, value: string}[]>([]);
const not_valid = ref(false);
const selectedFile = ref<File|null>(null);
const previewUrl = ref<string|null>();
const image_to_upload = ref<Blob>()
const { get_role, exit} = useUser();
const { get_avatar, update_avatar, get_avatars, update_avatar_from_blob } = useImage();
const selected_role = ref();
const role = get_role();
const {light_theme, dark_theme, get_current_theme} = useTheme();
const theme = ref(get_current_theme().value);
//const user_id = ref<string>();
//const password = ref<string>("");
//const is_new = ref<boolean>(false);
const change_theme = (value: Theme) =>
{
	if(value == 'dark')
		dark_theme();
	if(value == 'light')
		light_theme();
}
// const validate_password = (rule: FormItemRule, value: string): boolean => 
// {
//     return profile.value != null && value === profile.value.
    
// }
const validate_password = (rule: FormItemRule,value: string): boolean  => 
{
        if(profile.value)
		{
			if(props.new_user && profile.value.password.length < 1)
				return false;
			else return true
		}
		else return false;
}
const rules: FormRules =
{
    first_name: [
        {
            required: true,
            message: 'Необходимо ввести имя',
            trigger: ['input','blur', 'focus'],
        }
    ],
    second_name: [
        {
            required: true,
            message: 'Необходимо ввести отчество',
            trigger: ['input','blur', 'focus'],
        }
    ],
    surname: [
        {
            required: true,
            message: 'Необходимо ввести фамилию',
            trigger: ['input','blur', 'focus'],
        }
    ],
	username: [
        {
            required: true,
            message: 'Необходимо ввести имя пользователя',
            trigger: ['input','blur', 'focus'],
        }
    ],
	password: [
		{
			validator: validate_password,
			message: 'Необходимо ввести пароль',
            trigger: ['input','blur', 'focus'],
		}
    ],
    // reentered_password: [
    //     {
    //       required: true,
    //       message: 'Для аодтверждения введите пароль повторно',
    //       trigger: ['input', 'blur']
    //     },
    //     {
    //       validator: validate_password,
    //       message: 'Password is not same as re-entered password!',
    //       trigger: ['blur', 'password-input']
    //     },
    //   ]
}
const mouse_over = (e: MouseEvent) =>
{
    e.preventDefault()
    validate()
}
const validate = () =>
{
    formRef.value?.validate(
        (errors: Array<FormValidationError> | undefined) => 
        {
			if (!errors) 
			{
				not_valid.value = false;
			}
			else 
			{
				not_valid.value = true;
			}
        }
    )
}

const load_assotiated_employees = async (u: UserInfo) =>
{
	const list = await http_sevice.employees_service.get_employees_list();
	if(list)
	{
		employees_list.value = list.map(m=> 
		{
			if(m)
			return {
				label: `${m?.surname} ${m?.first_name} ${m?.second_name}`,
				value: m?.id
			}
			else
				return {
			label: '',
			value: ''
			}
		});
	}
	if(u.id.length > 0)
	{
		const ass = await http_sevice.employees_service.get_assotiate_employee(u.id);
		if(ass)
		{
			assotiated_employee.value = ass;
			selected_assotiated_employee.value = ass.id;
		}
	}
}

watch(() => props.profile, async (n) =>
{
    if(n)
	{
		load_assotiated_employees(n);
		profile.value = {
			first_name: n.first_name,
			second_name: n.second_name,
			surname: n.surname,
			role: n.role,
			permissions: n.permissions,
			password: "",
			id: n.id,
			token: n.token,
			username: n.username
		}
		previewUrl.value = await get_avatar(n.id) as string;
		validate();
	}
}, 
{immediate: true})
onUnmounted(()=>
{
	if(previewUrl.value)
		URL.revokeObjectURL(previewUrl.value)
})
const handleFileUpload = async (event: Event) => 
{
    const target = event.target as HTMLInputElement;
    const file = target.files?.item(0);
    if (!file) return;
    selectedFile.value = file as File;
    image_to_upload.value = await compressImage(selectedFile.value as File, {quality: 0.5, maxHeight: 200, maxWidth: 200, mimeType: 'image/webp'})
    // Создаем превью
    previewUrl.value =  URL.createObjectURL(image_to_upload.value);
};

const save_assotiated_employee = async (user_id: string) =>
{
	if(selected_assotiated_employee.value)
	{
		await http_sevice.employees_service.assotiate_employee_with_user(user_id, selected_assotiated_employee.value);
	}
}

const create_user = async () => 
{
	const new_user : CreateUserPayload = {
		username: profile.value?.username ?? "",
		first_name: profile.value?.first_name ?? "",
		second_name: profile.value?.second_name ?? "",
		password: profile.value?.password.length == 0 ? undefined : profile.value?.password,
		surname: profile.value?.surname ?? "",
		role: profile.value?.role,
		permissions: profile.value?.permissions
	}
	const formData = new FormData();
	if(image_to_upload.value && profile.value?.id)
	{
		formData.append('avatar', image_to_upload.value);
		update_avatar_from_blob(profile.value?.id, image_to_upload.value);
	}
	formData.append("user_info", JSON.stringify(new_user));
	const result = async () => 
	{
		console.log(is_all_permissions);
		if(is_all_permissions.value)
		{
			return await http_sevice.user_service.create_user_by_admin(formData, profile.value?.username as string);
		}
		else
		{
			return await http_sevice.user_service.create_user(formData, profile.value?.username as string);
		}

	}
	const uid = await result();
	if(uid)
	{
		//выставляем обратно на false
		emits('update:new_user', false);
		save_assotiated_employee(uid)
	}
}



const update_user = async () => 
{
	const user : UserInfoUpdate = 
	{
		id: profile.value?.id ?? "",
		first_name: profile.value?.first_name ?? "",
		second_name: profile.value?.second_name ?? "",
		password: profile.value?.password.length == 0 ? undefined : profile.value?.password,
		surname: profile.value?.surname ?? "",
	}
	const formData = new FormData();
	if(image_to_upload.value && profile.value?.id)
	{
		formData.append('avatar', image_to_upload.value);
		update_avatar_from_blob(profile.value?.id, image_to_upload.value);
	}
	formData.append("user_info", JSON.stringify(user));
	await http_sevice.user_service.profile_update(formData, profile.value?.username as string);
	save_assotiated_employee(profile.value?.id as string);
}

const update_user_by_admin = async () => 
{
		const user : AdminUserInfoUpdate = 
		{
			id: profile.value?.id ?? "",
			first_name: profile.value?.first_name ?? "",
			second_name: profile.value?.second_name ?? "",
			password: profile.value?.password.length == 0 ? undefined : profile.value?.password,
			surname: profile.value?.surname ?? "",
			role: profile.value?.role ?? roles[1],
			permissions: profile.value?.permissions ?? [permissions[3]]
		}
		const formData = new FormData();
		if(image_to_upload.value && profile.value?.id)
		{
			formData.append('avatar', image_to_upload.value);
			update_avatar_from_blob(profile.value?.id, image_to_upload.value);
		}
		formData.append("user_info", JSON.stringify(user));
		await http_sevice.user_service.admin_profile_update(formData, profile.value?.username as string);
		save_assotiated_employee(profile.value?.id as string);

}

const save = async () => 
{
	console.log(props.new_user);
    if (!selectedFile.value && !profile.value && !image_to_upload.value) return;
    if(props.new_user)
	{
		await create_user();
	}
	else
	{
		if(is_all_permissions.value)
		{
			await update_user_by_admin();
			
		}
		else
		{
			await update_user();
		}
	}
};
</script>
    
<style lang="scss" scoped>
$height: 700px;
.profile
{
    display: flex;
    flex-direction: column;
    align-items: center;
    overflow: 'hidden';
    overflow-y: 'hidden';
    
}
.ld
{
    width: 100%;
    margin-top: -5px;
}
.main
{
    display: flex;
    flex-direction: row;
}
.left
{
    display: flex;
    flex-direction: column;
    width: 400px;
    height: $height;
}
.right
{
    display: flex;
    flex-direction: column;
    width: 300px;
    height: $height;
}
.save-button
{
    width: 200px;
	align-self: center;
    margin-top: 10px;
}
.theme-buttons
{
    margin-bottom: 10px;
}
.ava-img
{
    cursor: pointer;
	max-width: 200px;
}
.avatar-form
{
	justify-items: center;
}
</style>