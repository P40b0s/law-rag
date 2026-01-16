<template lang="pug">
n-modal(v-model:show="props.is_open" :mask-closable="false")
	n-card(
		closable
		style="width: 600px; min-height: 600px;"
		title="Загрузка файлов в пакет"
		:bordered="false"
		@close="emits('update:is_open', false)"
		size="huge"
		role="dialog"
		aria-modal="true")
		template(#header-extra v-if="upload_statistic")
			n-statistic(label="Всего / Загружено / Ошибки")
				.statistic
					.statistic-element
						n-icon(color="white")
							ChartBar
						span {{upload_statistic.overall}}
					.statistic-element
						n-icon(color="green")
							CloudUpload
						span {{upload_statistic.uploaded}}  
					.statistic-element
						n-icon(color="red")
							FaceDizzy
						span {{upload_statistic.error}}
		file-uploader(v-if="props.packet_id" v-model:statistic="upload_statistic" :packet_id="props.packet_id" :multiple="props.multiple" :before_upload="props.before_upload")
</template>

<script lang="ts">
import { ref, type Component, watch, inject, onMounted, onUnmounted, computed, onBeforeUnmount, h, toRefs, type CSSProperties } from 'vue';
import { type Events, type Emitter } from '../services/emitter';
import { NForm, NFormItem, NInput, NModal, NList, NListItem, NCheckbox, NStatistic, NThing, NTooltip, NIcon, NCard, NButton, darkTheme, NTable, NScrollbar, NProgress, type UploadFileInfo} from 'naive-ui';
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

</script>
<script lang="ts" setup>
interface Props 
{
    is_open: boolean,
    packet_id?: string,
	multiple?: boolean,
	before_upload?(data: {file: UploadFileInfo, fileList: UploadFileInfo[]}): boolean
}
const props = defineProps<Props>();
const emits = defineEmits<{
    (e: 'update:is_open', value: boolean): void
}>()
const emitter = inject<Emitter<Events>>('emitter') as Emitter<Events>;
const upload_statistic = ref<UploadStatistic>();
const show_file_uploader = ref(false);
const selected_packet_id_for_uploader = ref<string>();
const selected_packet_id_for_file_viewer = ref<string>();
const {visible} = useVisible();
const show_uploader = (packet_id: string) =>
{
	selected_packet_id_for_uploader.value = packet_id;
	upload_statistic.value = undefined;
	show_file_uploader.value = true;
}

watch(show_file_uploader, async (n)=>
{
	if(n == false && selected_packet_id_for_uploader.value)
	{
		const packet = await http_sevice.get_packet(selected_packet_id_for_uploader.value);
		if(packet)
		{
			let index = packets.value.findIndex(f=>f.id == packet.id);
			if(index >= 0)
			{
				packets.value[index] = packet;
			}
		}
	}
})

const on_delete_packet = (id: string) =>
{
	let index = packets.value.findIndex(f=>f.id == id);
	packets.value.splice(index, 1);
}

const delete_packet = async (packet_id: string) =>
{
	await http_sevice.delete_packet(packet_id);
}
const uploaded_directory_name = ref("");
const packets = ref<Packets>([]);
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
const element_is_visible = visible(['Administrator', 'User'], ['All'])
onMounted(async () => 
{
	emitter.on('delete_packet', on_delete_packet);
	packets.value = await http_sevice.get_packets();
});
onUnmounted(() => emitter.off('delete_packet', on_delete_packet));

const upload_files_handler = (packet_id: string) =>
{

}

const get_files_handler = async (packet_id: string) =>
{
	selected_packet_id_for_file_viewer.value = packet_id;
}

const edit_handler = async (user_id: number) =>
{
	
}

</script>
	
<style lang="scss" scoped>

.statistic
{
	display: flex;
	flex-direction: row;
	align-items: center;
	gap: 28px;
}
.statistic-element
{
	display: flex;
	flex-direction: row;
	align-items: center;
	gap: 3px;
}

</style>