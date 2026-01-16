<template lang="pug">
.uploader
	n-upload(:multiple="props.multiple" 
		abstract
		:show-remove-button="false"
		:custom-request="customRequest"
		v-model:file-list="file_list"
		@update:file-list="update"
		list-type="image"
		:render-icon="file_icon"
		@before-upload="before_upload")
		n-upload-trigger(#="{ handleClick }" abstract)
			n-button(@click="handleClick") Выбрать файлы
		n-progress(
			type="line"
			:show-indicator="false"
			status="success"
			:percentage="percentage")
		n-scrollbar(style="max-height: 600px; margin-top: 5px;")
			n-upload-file-list
</template>
        
<script lang="ts">
import {UserAvatarFilledAlt} from '@vicons/carbon'
import { defineComponent, ref, onMounted, watch, computed, h, type CSSProperties} from 'vue'
import {NTooltip, NAvatar, NIcon, NCheckbox, NInput, NProgress, NSelect, NCard, NScrollbar, NUploadFileList, NUploadTrigger, NUpload, NButton, type SelectOption, type UploadCustomRequestOptions, type UploadFileInfo, type UploadSettledFileInfo} from 'naive-ui'
import { type Document } from "@/types/document"
import useUser from '@composables/useUser'
import { Error } from '@vicons/carbon'
import { base64_to_uint8_array } from '@/services/helpers'
import { type Permission, permissions } from '@/types/permission'
import { match } from 'ts-pattern'
import { roles, type Role } from '@/types/user_role'
import { http_sevice } from '@/services/http_service/http_service';
import { notify_service } from '@/services/notification_service'
import { api_path } from '@/services/http_service/http_client'
import {homer_ico, pdf_ico, msoffice_ico} from '@/services/svg';
import { uuidv7 } from 'uuidv7'
import { type UploadStatistic } from '@/types/upload_statistic'
</script>

<script lang="ts" setup>
interface Props 
{
    statistic?: UploadStatistic,
    packet_id: string,
	multiple?: boolean,
	before_upload?(data: {file: UploadFileInfo, fileList: UploadFileInfo[]}): boolean
}
const props = defineProps<Props>();
const emits = defineEmits<{
    (e: 'update:statistic', counts: UploadStatistic): void
}>()
const {get_role,  get_token, get_role_name} = useUser();
const current_role = get_role();
const selectedFiles = ref<File[]>([]);
const progress = ref(0);
const percentage = computed(()=> 
{
	return count.value.uploaded / (count.value.overall / 100);
})
const file_list = ref<UploadFileInfo[]>([]);
// const count = computed(()=> 
// {
// 	return {
// 		uploaded: file_list.value.filter(m=>m.status == 'finished').length,
// 		error: file_list.value.filter(m=>m.status == 'error').length,
// 		overall: file_list.value.length
// 	} as UploadStatistic
// })
// const update = (fileList: UploadFileInfo[]) =>
// {
// 	emits('update:statistic', {
// 		uploaded: fileList.filter(m=>m.status == 'finished').length,
// 		error: fileList.filter(m=>m.status == 'error').length,
// 		overall: fileList.length
// 	} as UploadStatistic)
// }
const count = ref<UploadStatistic>({uploaded: 0, error: 0, overall: 0});

const update = (fileList: UploadFileInfo[]) =>
{
	count.value = {
		uploaded: fileList.filter(m=>m.status == 'finished').length,
		error: fileList.filter(m=>m.status == 'error').length,
		overall: fileList.length
	} as UploadStatistic;
	emits('update:statistic', count.value)
}

const get_icon = (icon: string) =>
{
	return h(NAvatar, 
	{
		src: icon,
		style:
		{
			background: 'transparent'
		} as CSSProperties
	})
}
const file_icon = (file: UploadSettledFileInfo) =>
{
	if (!file)
		return undefined
	return match(file.type)
	.with('application/pdf', () => get_icon(pdf_ico))
	.with('application/msword', () => get_icon(msoffice_ico))
	.with('application/vnd.openxmlformats-officedocument.wordprocessingml.document', () => get_icon(msoffice_ico))
	.with('application/rtf', () => get_icon(msoffice_ico))
	.otherwise(() => undefined)
}
const before_upload = (data: {file: UploadFileInfo, fileList: UploadFileInfo[]}) =>
{
	const accepted = [
	'application/msword'
	,'application/pdf'
	,'application/vnd.openxmlformats-officedocument.wordprocessingml.document'
	,'application/rtf'];
	let result = false;
	if(data.file.file?.type)
	{
		if(accepted.includes(data.file.file?.type))
			result = true
		else
		{
			notify_service.notify_warning("Тип файла не поддерживается",
			 "Загрузить можно только документы в форматах *.pdf, *.doc, *.docx, *.rtf")
			 result = false;
		}	
	}
	else 
	{
		notify_service.notify_warning("Тип файла не поддерживается",
		 "Загрузить можно только документы в форматах *.pdf, *.doc, *.docx, *.rtf")
		 result = false;
	}
	if(props.before_upload)
		return props.before_upload(data) && result
	return result;
}
const customRequest = ({
      file,
      data,
      headers,
      withCredentials,
      action,
      onFinish,
      onError,
      onProgress
    }: UploadCustomRequestOptions) => 
{
	const formData = new FormData()
	formData.append('file', file.file as File);
	const xhr = new XMLHttpRequest();
	xhr.upload.addEventListener('progress', (e) => {
	if (e.lengthComputable) 
	{
		onProgress({ percent: Math.round((e.loaded / e.total) * 100) })
		//console.log(`${file.name}: ${Math.round((e.loaded / e.total) * 100)}%`);
	}
	});

	xhr.addEventListener('loadend', () => 
	{
		if (xhr.status >= 200 && xhr.status < 300) 
		{
			onFinish();
			console.log(`Файл ${file.name} успешно загружен`);
		} 
		else 
		{
			console.log(`Ошибка загрузки файла ${file.name} ${ xhr.statusText || 'Unknown error'}`);
			onError()
		}
	});
	xhr.open('POST', api_path + 'packets/upload/' + props.packet_id, true);
	xhr.setRequestHeader("Authorization",  "Bearer " + get_token());
	xhr.send(formData);
}
</script>
    
<style lang="scss" scoped>
.upload-item {
  margin: 1rem 0;
  padding: 0.5rem;
  border: 1px solid #ddd;
}
.progress-text 
{
  margin-left: 1rem;
}
.completed 
{
  opacity: 0.7;
}
.files-list
{
	margin-right: 7px;
	width: 100%;
}
.uploader
{
	width: 100%;
	display: flex;
	flex-direction: column;
	gap:5px;
}
.scrollbar
{
	margin-top: 50px;
	max-height: 50px !important;
}
</style>