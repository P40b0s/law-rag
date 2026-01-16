<template lang="pug">
div
	input(type="file" multiple @change="handleFileSelect")
	button(@click="uploadAll") Загрузить 
	div.upload-item(v-for="(progress, fileId) in uploadState" :key="fileId")
		div.file-info {{ progress.file.name }} 
			span(v-if="progress.completed") ✅
			span(v-else-if="progress.error") ❌ {{ progress.error }}
		progress( 
		:value="progress.loaded" 
		:max="progress.total"
		:class="{ 'completed': progress.completed }")
		span.progress-text {{ Math.round((progress.loaded / progress.total) * 100) }}%
</template>
        
<script lang="ts">
import {UserAvatarFilledAlt} from '@vicons/carbon'
import { defineComponent, ref, onMounted, watch, computed} from 'vue'
import {NTooltip, NAvatar, NIcon, NCheckbox, NSelect, type SelectOption} from 'naive-ui'
import { type Document } from "@/types/document"
import useUser from '@composables/useUser'
import { Error } from '@vicons/carbon'
import { base64_to_uint8_array } from '@/services/helpers'
import { match } from 'ts-pattern'
import { roles, type Role } from '@/types/user_role'
import { http_sevice } from '@/services/http_service/http_service';
import { notify_service } from '@/services/notification_service'
import { api_path } from '@/services/http_service/http_client'
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
//const props = defineProps<Props>();
const emits = defineEmits<{
    (e: 'update:value', role: Role): void
}>()
const {get_role, get_role_name} = useUser();
const current_role = get_role();
const selectedFiles = ref<File[]>([]);
const progress = ref(0);

// const handleFileSelect = (event: Event) => 
// {
//   const input = event.target as HTMLInputElement;
//   if (input.files) 
//   {
//     selectedFiles.value = Array.from(input.files);
//   }
// }

// const uploadFiles = async () => 
// {
//   if (selectedFiles.value.length === 0) return;
//   progress.value = 0;
//   try 
//   {
//     const result = await http_sevice.upload_files(selectedFiles.value, (percent) => 
// 	{
//       progress.value = percent;
//     });
//     notify_service.notify_success("Файлы успешно загружены", "Загружено " + result.count + " файлов")
//     console.log('Upload successful:', result);
//   } 
//   catch (error) 
//   {
//     console.error('Upload failed:', error);
//     notify_service.notify_error("Ошибка загрузки файлов", error as string);
//   } 
//   finally 
//   {
//     progress.value = 0;
//   }
// }
interface FileUploadProgress 
{
  file: File;
  loaded: number;
  total: number;
  completed: boolean;
  error?: string;
}
const { get_token } = useUser();
const uploadState = ref<Record<string, FileUploadProgress>>({});
const handleFileSelect = (e: Event) => 
{
  const input = e.target as HTMLInputElement;
  if (input.files) 
  {
    uploadFiles(input.files);
  }
}
const uploadFiles = (files: FileList) => 
{
  Array.from(files).forEach(file => 
  {
    const fileId = `${file.name}-${file.lastModified}`;
    uploadState.value[fileId] = 
	{
      file,
      loaded: 0,
      total: file.size,
      completed: false
    };

    const xhr = new XMLHttpRequest();
    const formData = new FormData();
    formData.append('file', file);

    xhr.open('POST', api_path + 'packets/upload', true);
	xhr.setRequestHeader("Authorization",  "Bearer " + get_token());
    // Отслеживаем прогресс для конкретного файла
    xhr.upload.addEventListener('progress', (e) => 
	{
      if (e.lengthComputable) 
	  {
        uploadState.value[fileId] = 
		{
          ...uploadState.value[fileId],
          loaded: e.loaded,
          total: e.total
        };
      }
    });

    xhr.addEventListener('loadend', () => 
	{
      if (xhr.status >= 200 && xhr.status < 300) 
	  {
        uploadState.value[fileId] = 
		{
          ...uploadState.value[fileId],
          completed: true
        };
        console.log(`Файл ${file.name} успешно загружен`);
      } 
	  else 
	  {
        uploadState.value[fileId] = 
		{
          ...uploadState.value[fileId],
          error: xhr.statusText || 'Unknown error'
        };
      }
    });
    xhr.send(formData);
  });
}
</script>
    
<style lang="scss" scoped>
.upload-item {
  margin: 1rem 0;
  padding: 0.5rem;
  border: 1px solid #ddd;
}
.progress-text {
  margin-left: 1rem;
}
.completed {
  opacity: 0.7;
}
</style>