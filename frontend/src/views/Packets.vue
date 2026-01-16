<template lang="pug">
div.table-root
	files-uploader-modal(v-model:is_open="show_file_uploader" :packet_id="selected_packet_id_for_uploader" :multiple="true")
	n-table.tbl(:bordered="false" :single-line="false")
		thead
			tr
				th.created-header Дата
				th.dir-header Директория
				th.count-header Файлов
				th.avatar-header Юзер
				th.buttons-header
					.actions-header
						div Действия
						n-tooltip(v-if="element_is_visible") Добавить новый пакет
							template(#trigger)
								n-button( quaternary circle )
									template(#icon)
										svg-icon(:svg="plus_ico" @click="add_packet" :size="34")
		tbody.scrollable-tbody
			template(v-for="p in packets")
				tr
					td.created-body {{p.created.to_string(DateFormat.DateTime)}}
					td.dir-body {{p.packet_dir}}
					td.count-body {{p.files_count}}
					td.avatar-body
						.avatar
							user-icon(:user_id="p.user_id" :size="60" show_name)
					td.buttons-body
						.buttons
							n-button(@click="get_files_handler(p.id)") Список Файлов
							n-button(v-if="element_is_visible" @click="show_uploader(p.id)") Загрузить
							n-button(v-if="element_is_visible" @click="delete_packet(p.id)") Удалить
	.files
		//- suspense не работал пока не поставил key...похоже что-то кешируется....
		suspense(:key="key_for_file_viewer")
			template(#default)
				files-viewer-async(
					v-if="selected_packet_id_for_file_viewer"
					:packet_id="selected_packet_id_for_file_viewer")
			template(#fallback)
				loader(status="Загрузка файлов..." key="loader")
		
		
				
</template>

<script lang="ts">
import { ref, defineAsyncComponent, type Component, watch, inject, onMounted, onUnmounted, computed, onBeforeUnmount, h, toRefs, type CSSProperties } from 'vue';
import { type Events, type Emitter } from '../services/emitter';
import { NForm, NFormItem, NInput, NModal, NList, NListItem, NCheckbox, NStatistic, NThing, NTooltip, NIcon, NCard, NButton, darkTheme, NTable, NScrollbar, NProgress} from 'naive-ui';
import { notify_service } from '@/services/notification_service';
import useUser from '@/composables/useUser';
import {Upload} from '@vicons/carbon'
import { pdf_ico, plus_ico, msoffice_ico, warning_ico, success_ico, doc_error_ico } from '@/services/svg';
import SvgIcon from '@/components/SvgIcon.vue';
//import  user_service  from '../services/user_service';
import UserIcon from '@components/UserIcon.vue';
import { match } from 'ts-pattern';
import { type Role, roles } from '@/types/user_role';
import {type UserInfo} from '@/types/user_info';
import ProfileEditor from '@/components/ProfileEditor.vue'
import FileUploader from '@/components/FileUploader.vue';
import { http_sevice } from '@/services/http_service/http_service';
import { type UploadStatistic } from '@/types/upload_statistic';
import {FetchUploadCloud, FaceDizzy, CloudUpload, ChartBar, DocumentDownload} from '@vicons/carbon'
import {DateFormat} from '@services/date'
import { type Packets, type Packet, type PacketFiles, type FilesWithDocument, type FileWithDocument, type PacketFile } from '@/types/packet';
import {AsyncFileDocumentMap} from '@/components/file_document_map';
import useVisible from '@/composables/useVisible';
import StatusesList from '@/components/documents/StatusesList.vue';
import FilesViewer from '@/components/FilesViewer.vue';
import { type Document } from '@/types/document';
import FilesUploaderModal from '@/components/FilesUploaderModal.vue';
import Loader from '@/components/Loader.vue';
const FilesViewerAsync = defineAsyncComponent({
    loader: () => import('@/components/FilesViewer.vue')
  })
</script>
<script lang="ts" setup>

const emitter = inject<Emitter<Events>>('emitter') as Emitter<Events>;
const selected_packet_id_for_uploader = ref<string>();
const show_file_uploader = ref<boolean>(false);
const selected_packet_id_for_file_viewer = ref<string>();
const key_for_file_viewer = ref(false);
const packets = ref<Packets>([]);
const {visible} = useVisible();
const element_is_visible = visible(['Administrator', 'User'], ['All'])
const show_uploader = (packet_id: string) =>
{
	selected_packet_id_for_uploader.value = packet_id;
	show_file_uploader.value = true;
}

const get_files_handler = (packet_id: string) =>
{
	selected_packet_id_for_file_viewer.value = packet_id;
	if(selected_packet_id_for_file_viewer.value)
		key_for_file_viewer.value = !key_for_file_viewer.value;
}

const on_delete_packet = (id: string) =>
{
	let index = packets.value.findIndex(f=>f.id == id);
	packets.value.splice(index, 1);
}

const delete_packet = async (packet_id: string) =>
{
	await http_sevice.delete_packet(packet_id);
}
const add_packet = async () =>
{
	const packet = await http_sevice.add_packet();
	if(packet)
	{
		packets.value.splice(0, 0, packet);
	}
	else
	{	
		notify_service.notify_error("Ошибка добавления пакета", "");
	}
}

onMounted(async () => 
{
	emitter.on('delete_packet', on_delete_packet);
	packets.value = await http_sevice.get_packets();
});
onUnmounted(() => emitter.off('delete_packet', on_delete_packet));
watch(show_file_uploader, async (n)=>
{
	if(!n && selected_packet_id_for_uploader.value)
	{
		let index = packets.value.findIndex(f=>f.id == selected_packet_id_for_uploader.value);
		const p = await http_sevice.get_packet(selected_packet_id_for_uploader.value);
		if(p)
			packets.value.splice(index, 1, p);
		if(selected_packet_id_for_file_viewer.value)
		{
			if(selected_packet_id_for_file_viewer.value != selected_packet_id_for_uploader.value)
			{
				selected_packet_id_for_file_viewer.value = selected_packet_id_for_uploader.value;
			}
			key_for_file_viewer.value = !key_for_file_viewer.value;
		}
	}
})
</script>
    
<style lang="scss" scoped>
$created-width: 60px;
$dir-width: 100px;
$count-width: 50px;
$avatar-width: 60px;
$buttons-width: 80px;
.table-root
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
.files
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

.dir-header
{
	width: $dir-width;
	height: 50px;
	text-align: center;
}
.created-header
{
	width: $created-width;
	height: 50px;
	text-align: center;
}
.count-header
{
	width: $count-width;
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
.created-body
{
	width: $created-width;
	height: 50px;
	text-align: center;
}
.dir-body
{
	width: $dir-width;
	height: 50px;
	text-align: center;
}
.count-body
{
	width: $count-width;
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
  	max-height: calc(100vh - 50px);
}
.new-button
{
	align-self: stretch;
}
.slide-fade-enter-active {
  transition: all 0.9s ease-out;
}

.slide-fade-leave-active {
  transition: all 0.9s cubic-bezier(1, 0.5, 0.8, 1);
}

.slide-fade-enter-from,
.slide-fade-leave-to {
  transform: translateY(10px);
  opacity: 0;
}
</style>