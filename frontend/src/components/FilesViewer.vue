<template lang="pug">
n-list.files-list(hoverable v-if="files")
	files-uploader-modal(v-model:is_open="show_file_uploader" :packet_id="props.packet_id" :before_upload="before_upload")
	template(#header)
		div.header
			n-tooltip Выбрать все файлы
				template(#trigger)
					n-checkbox(@update:checked="check_all" size="large") Выбрано: {{checked_count}}/{{Object.entries(files).length}} документов
			n-spin(v-if="copy_in_progress")
			n-tooltip Выгрузить файлы для последующей загрузки в комплекс
				template(#trigger)
					n-button(:disabled="checked_count == 0 || copy_in_progress" text @click="copy_files")
						template(#icon)
							n-icon(:component="Upload" :size="35" color="green")
	.scrollbar
		n-list-item(v-for="(files_group, name) in files" :key="name" :class="{ checked: files_group.checked }")
			template(#prefix)
				.prefix-checkbox
					n-checkbox(v-model:checked="files_group.checked" size="large" :disabled="!files_group.have_all_files")
					
			template(#suffix)
				.suffix-panel
					component(:is="render_postfix(files_group)")
					component(:is="render_document(files_group)")
			n-thing
				template(#header)
					div {{name}}
				template(#action)
					.action-template
						n-tooltip Добавить файл
							template(#trigger)
								svg-icon.extension-icon(:size="34" :svg="upload_ico" @click="upload_file(name)")
						component(v-for="f in files_group.grouped_files" :is="render_extension(f.file)")
						
					
</template>
        
<script lang="ts">
import {UserAvatarFilledAlt} from '@vicons/carbon'
import { defineComponent, ref, onMounted, watch, computed, h, type CSSProperties} from 'vue'
import { NForm, NFormItem, NInput, NModal, NList, NSwitch, NSpin, NListItem, NCheckbox, NStatistic, NThing, NTooltip, NIcon, NCard, NButton, darkTheme, NTable, NScrollbar, NProgress, type UploadFileInfo} from 'naive-ui';
import { type Document } from "@/types/document"
import useUser from '@composables/useUser'
import { Error, Upload } from '@vicons/carbon'
import { UploadFileSharp } from '@vicons/material'
import { base64_to_uint8_array, sleepNow } from '@/services/helpers'
import { type Permission, permissions } from '@/types/permission'
import { match } from 'ts-pattern'
import { roles, type Role } from '@/types/user_role'
import { http_sevice } from '@/services/http_service/http_service';
import { notify_service } from '@/services/notification_service'
import FilesUploaderModal from './FilesUploaderModal.vue';
import { api_path } from '@/services/http_service/http_client'
import {homer_ico, pdf_ico, msoffice_ico, doc_error_ico, success_ico, warning_ico, upload_ico} from '@/services/svg';
import { uuidv7 } from 'uuidv7'
import { type UploadStatistic } from '@/types/upload_statistic'
import { type PacketFile, type FilesWithDocument } from '@/types/packet'
import StatusesList from '@/components/documents/StatusesList.vue';
import SvgIcon from '@/components/SvgIcon.vue';
type FilesList =
{
	grouped_files: FilesWithDocument
	checked: boolean,
	found_in_complex: boolean,
	have_all_files: boolean
}
</script>

<script lang="ts" async setup>

interface Props 
{
    packet_id: string,
}
const props = defineProps<Props>();
const files = ref<Record<string, FilesList>>()
const show_file_uploader = ref(false);
const uploaded_filename = ref("");
const got_all_files = (files: FilesWithDocument) => 
	files.some(f=>f.file.extension == "pdf") && 
	files.some(f=>f.file.extension == "docx" || f.file.extension == "doc" || f.file.extension == "rtf")
const on_load = async () =>
{
	const fl = await http_sevice.get_files(props.packet_id);
	const grouped = fl.reduce((acc: Record<string, FilesList>, file) => 
	{
        const key = file.file.name;
		const current_checked_state = files.value ? (files.value[key] ? files.value[key].checked : false) : false;
        if (!acc[key]) 
		{
            acc[key] = {
				grouped_files: [],
				checked: current_checked_state,
				found_in_complex: false,
				have_all_files: false
			};
        }
        acc[key].grouped_files.push(file);
		acc[key].found_in_complex = file.document !== null;
		acc[key].have_all_files = got_all_files(acc[key].grouped_files)
        return acc;
    }, {});
	files.value = grouped;
}

const {get_token} = useUser();
await on_load();

const upload_file = (filename: string) =>
{
	uploaded_filename.value = filename;
	show_file_uploader.value = true;
}
const before_upload = (data: {file: UploadFileInfo, fileList: UploadFileInfo[]}): boolean => 
{
	const lastDotIndex = data.file.name.lastIndexOf('.');
    const name = lastDotIndex === -1 ? data.file.name : data.file.name.substring(0, lastDotIndex);
	if(name != uploaded_filename.value)
	{
		notify_service.notify_error("Имя файла отличается", "Имя файла должно совпадать с файловым пакетом в который произодиться загрузка файла");
		return false;
	}
	return true;
}
const checked_count = computed(() =>
{
	if(files.value)
	{
		const entries = Object.entries(files.value);
		return entries.filter(f=>f[1].checked).length;
	}
	else return 0;
})
const check_all = (value: boolean) =>
{
	if(files.value)
	{
		if(!value)
		{
			const entries = Object.entries(files.value);
			entries.forEach(f=>f[1].checked = false)
			
		}
		else
		{
			const entries = Object.entries(files.value);
			entries.forEach(f=>
			{
				if(f[1].have_all_files)
					f[1].checked = true;
			})
		}
	}
}
const render_document = (value: FilesList) =>
{
	if(value.found_in_complex)
	{
		const doc = value.grouped_files.find(f=>f.document !== null)?.document as Document; 
		return h(StatusesList,
		{
			redaction: doc.complex_information.redaction,
			checked: doc.checked
		})
	}
	return h(NTooltip,
		{
			style:
			{
				maxWidth: '250px'
			} as CSSProperties
		},
		{

			trigger:()=> h(SvgIcon, 
			{
				svg: doc_error_ico,
				highlight_size: 7,
				size: 26
			}),
			default:() => "Не найдено соответствие загруженного образа pdf и образа документа в базе комплекса, документ еще не загружен в базу данных комплекса"
		}) 	
}
const copy_in_progress = ref(false);
const copy_files = async () =>
{
	if(files.value)
	{
		const entries = Object.entries(files.value);
		const fl = entries
		.filter(f=>f[1].checked)
		.flatMap(m=>m[1].grouped_files)
		.map(m=>m.file.hash);
		let cp = await http_sevice.copy_files_to_complex(props.packet_id, fl);
		if(cp)
		{
			notify_service.notify_success("Файлы скопированы", "Выбраные файлы скопированы, можно запускать импорт в приложении комплекс");
			check_all(false);
		}
		else
		{
			copy_in_progress.value = false;
		}
	}
}
const render_extension = (file: PacketFile) =>
{
	
	return h('div',
	{
		class: 'extension-icon',
		
	},
	h(NTooltip,
	{
		placement: 'top-end'
	},
	{
		trigger:() => h(SvgIcon, 
		{
			svg: (file.extension == "docx" || file.extension == "doc" || file.extension == "rtf") ? msoffice_ico : pdf_ico,
			size: 34,
			onClick: async () =>
			{
				const token = get_token();
				if(token)
				{
					const ok = await http_sevice.download_file(file.name, file.extension, file.hash, token);
					notify_service.notify_success("Загрузка завершена", "Файл " + file.name + "." + file.extension + " успешно загружен")
				}
					
			}
		}),
		default:() => (file.extension == "docx" || file.extension == "doc" || file.extension == "rtf") ? "Файл с текстом (скачать)" : "Файл образа документа (скачать)"
	}),
	)

}


const render_postfix = (files: FilesList) =>
{
	if(files.have_all_files)
		return h(SvgIcon, 
		{
			svg:  success_ico,
			size: 26
		})
	else
		return h(NTooltip,
		{
			style:
			{
				maxWidth: '250px'
			} as CSSProperties
		},
		{

			trigger:()=> h(SvgIcon, 
			{
				svg: warning_ico,
				highlight_size: 7,
				size: 26
			}),
			default:() => "Не все файлы готовы для выгрузки этого документа в комплекс, обработайте и догрузите недостающие файлы"
		}) 	
}
watch(show_file_uploader, async (n)=>
{
	if(!n)
	{
		await on_load();
	}
})
</script>
    
<style lang="scss" scoped>

.scrollbar
{
	display: block;
  	overflow-y: auto;
  	max-height: calc(100vh - 70px);
}
.extension-icon
{
	cursor: pointer;
	margin-right: 10px;
}
.prefix-checkbox
{
	display: flex;
	flex-direction: row;
	align-content: center;
	align-items: center;
}
.action-template
{
	display: flex;
	flex-direction: row;
}
.add-file-button
{
	margin-right: 8px;
}
.suffix-panel
{
	display: flex;
	flex-direction: column;
	align-items: center;
	gap: 10px;
}
.checked
{
	background-color: rgb(32, 79, 87);
}
.checked:hover
{
	--n-merged-color-hover:  rgb(41, 96, 105);
}
.header
{
	display: flex;
	flex-direction: row;
	justify-content: space-between;
}
.files-list
{
	width: 100%;
}

</style>