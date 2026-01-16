<template lang="pug">
.file-uploader
  //- Область для drag & drop
  .drop-area(
    @click="triggerFileInput"
    @drop="handleDrop"
    @dragover="handleDragOver"
    @dragleave="handleDragLeave"
    :class="{ 'drop-area--active': isDragOver }"
  )
    n-space(vertical align="center")
      n-icon(size="48" color="#2080f0")
        CloudUploadOutline
      n-text(style="font-size: 16px;") 
        | Перетащите файлы сюда или 
        n-text(color="#2080f0") нажмите для выбора
      n-text(depth="3" style="font-size: 12px;") 
        | Максимум {{ max_files }} файлов, каждый до {{ max_file_size_MB }}MB
    
    input(
      ref="fileInputRef"
      type="file"
      :multiple="multiple"
      :accept="accept"
      @change="handleFileSelect"
      style="display: none;"
    )

  //- Список выбранных файлов
  n-list(v-if="files.length > 0")
    n-list-item.file-item(v-for="file in files" :key="file.id + file.filename")
      n-thing(:title="file.filename" :description="formatFileSize(file.size)")
        template(#avatar)
          n-icon(:color="getFileIconColor(file.mime_type)" :size="40")
            component(:is="getFileIcon(file.mime_type)")
        template(#header-extra)
          n-space
            n-popconfirm(
                v-if="file.id.length > 0 || file.error"
                @positive-click="() => removeFile(file.id)")
                template(#trigger)
                    n-button(
                    text
                    type="error"
                    size="small"
                    )
                        template(#icon)
                            n-icon
                                TrashOutline
                        | Удалить
                div(v-if="file.file") Файл будет удален из списка загрузки
                div(v-else) Внимание! Файл будет удален c сервера навсегда! 
        template(#action v-if="!file.file")
            n-button(
                    text
                    type="success"
                    size="small"
                    @click="(e) => download_file_handler(file)"
                    )
                        template(#icon)
                            n-icon
                                Download
                        | Скачать
        template(#footer v-if="file.in_progress")
          n-progress(type="line" :percentage="file.percentage" :type="file.error ? error : success")

</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import {
  NSpace,
  NIcon,
  NText,
  NList,
  NListItem,
  NThing,
  NButton,
  NAlert,
  NProgress,
  NPopconfirm
} from 'naive-ui'
import {
  CloudUploadOutline,
  TrashOutline,
  DocumentOutline,
  ImageOutline,
  DocumentTextOutline,
  ArchiveOutline,
  VideocamOutline,
  MusicalNotesOutline,
  Download
} from '@vicons/ionicons5'
import { type TaskFile } from '@/types/task'
import { notify_service } from '@/services/notification_service'
import { useFileUpload } from '@/composables/useFileUpload'
import { http_sevice } from '@/services/http_service/http_service'

interface UploadProcess
{
  complete: boolean,
  percentage: number,
  in_progress: boolean,
  error?: string
}

interface Props {
  task_id?: string,
  multiple?: boolean
  max_files?: number
  max_file_size_MB?: number
  accept?: string
  files?: TaskFile[]
}

const props = withDefaults(defineProps<Props>(), {
  multiple: true,
  max_files: 50,
  max_file_size_MB: 200,
  accept: '*/*',
  files: () => []
})

const emit = defineEmits<{
  'update:files': [files: TaskFile[]]
  'files-change': [files: TaskFile[]]
}>()


const fileInputRef = ref<HTMLInputElement>()
const isDragOver = ref(false)
const files = ref<(TaskFile & UploadProcess)[]>([])

// Обработчик клика по области
const triggerFileInput = () => {
  fileInputRef.value?.click()
}

// Обработчик выбора файлов через input
const handleFileSelect = (event: Event) => 
{
  const target = event.target as HTMLInputElement
  const selectedFiles = target.files
  
  if (selectedFiles && selectedFiles.length > 0) 
  {
    processFiles(Array.from(selectedFiles))
    target.value = '' // Сбрасываем input
  }
}
const download_file_handler = async (file: TaskFile) =>
{
   const {download_file_async} = useFileUpload();
   await download_file_async(file.id, file.filename, `tasks/files/${file.id}`);
}

// Обработчик drag over
const handleDragOver = (event: DragEvent) => {
  event.preventDefault()
  isDragOver.value = true
}

// Обработчик drag leave
const handleDragLeave = (event: DragEvent) => {
  event.preventDefault()
  isDragOver.value = false
}

// Обработчик drop
const handleDrop = (event: DragEvent) => {
  event.preventDefault()
  isDragOver.value = false
  
  const droppedFiles = event.dataTransfer?.files
  if (droppedFiles && droppedFiles.length > 0) {
    processFiles(Array.from(droppedFiles))
  }
}

// Обработка добавленных файлов
const processFiles = async (fileList: File[]) => 
{
  // Проверка лимита файлов
  if (files.value.length + fileList.length > props.max_files) 
  {
    notify_service.notify_error(`Максимальное количество файлов: ${props.max_files}`)
    return
  }

  const newFiles: (TaskFile & UploadProcess)[] = []

  for (const file of fileList) 
  {
    // Проверка размера файла
    if (file.size > props.max_file_size_MB * 1024 * 1024) 
    {
      notify_service.notify_error(`Файл "${file.name}" превышает максимальный размер ${props.max_file_size_MB}MB`)
      continue
    }
    if (files.value.find(f=>f.filename == file.name)) 
    {
      notify_service.notify_error(`Файл с таким именем уже есть в списке "${file.name}"`)
      continue
    }

    // Создаем объект TaskFile
    const taskFile: TaskFile & UploadProcess = {
      id: "",
      task_id: props.task_id ?? "",
      filename: file.name,
      storage_path: '', // Можно сгенерировать путь или оставить пустым до загрузки
      size: file.size,
      mime_type: file.type || 'application/octet-stream',
      file: file,
      hash: "",
      complete: false,
      percentage: 0,
      in_progress: false,
    }
    newFiles.push(taskFile)

  }

  if (newFiles.length > 0) 
  {
    files.value = [...files.value, ...newFiles]
    
    notify_service.notify_success(`Добавлено файлов: ${newFiles.length}`)
  }
  //upload_files(files.value.filter(f=> newFiles.map(m=> m.id).includes(f.id)));
  await upload_files()
}
const upload_files = async () =>
{
  const {upload_file_async} = useFileUpload()
  for(const file of files.value)
  {
    if(file.file)
    {
      file.in_progress = true;
      await upload_file_async(`tasks/add_file/${props.task_id}`, file.file,
        {
          on_progress_update(percent) 
          {
            file.percentage = percent;
          },
          on_complete() 
          {
            file.in_progress = false;
            file.complete = true;
            notify_service.notify_success(`Файл "${file.file?.name}" успешно загружен на сервер`)
            file.file = undefined;
          },
          on_error(error)
          {
            file.in_progress = false;
            file.error = error;
            notify_service.notify_error(`Ошибка загрузки файла "${file.file?.name}"`, error);
          }
        }
      )
    }
  }
  if(props.task_id)
  {
    //если есть таск id то обновляем список файлов с сервера и они автоматически обновляются в этом компоненте
    const files = await http_sevice.tasks_service.get_files(props.task_id);
    emit('update:files', files)
    emit('files-change', files)
  }

  //emitFileChange();
}

//TODO организовать удаление файла на бэке
const removeFile = async (id: string) => 
{
  files.value = files.value.filter(f => f.id !== id)
  await http_sevice.tasks_service.delete_file(id)
  emitFileChange()
}

// Очистка всех файлов
const clearAll = () => 
{
  files.value = []
  emitFileChange()
  notify_service.notify_warning('Все файлы удалены')
}

// Вспомогательные функции
const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const getFileIcon = (mimeType: string) => {
  if (mimeType.startsWith('image/')) return ImageOutline
  if (mimeType.startsWith('video/')) return VideocamOutline
  if (mimeType.startsWith('audio/')) return MusicalNotesOutline
  if (mimeType.includes('pdf') || mimeType.includes('document')) return DocumentTextOutline
  if (mimeType.includes('zip') || mimeType.includes('archive')) return ArchiveOutline
  return DocumentOutline
}

const getFileIconColor = (mimeType: string): string => 
{
  if (mimeType.startsWith('image/')) return '#ff6b6b'
  if (mimeType.startsWith('video/')) return '#4ecdc4'
  if (mimeType.startsWith('audio/')) return '#45b7d1'
  if (mimeType.includes('pdf')) return '#e74c3c'
  if (mimeType.includes('document')) return '#3498db'
  if (mimeType.includes('zip')) return '#9b59b6'
  if (mimeType.includes('rar')) return '#9b59b6'
  return '#95a5a6'
}

const emitFileChange = () => {
  emit('update:files', files.value)
  emit('files-change', files.value)
}

// Watch для внешних изменений modelValue
watch(() => props.files, (newValue) => 
{
  //if (newValue !== files.value) 
  //{
    files.value = newValue.map(f=>
      {
        return {
          ...f,
          in_progress: false,
          complete: false,
          percentage: 0,
        }
      }
    )
  //}
}, { deep: true, immediate: true })

watch(files.value, (new_value) => 
{
  
  
}, { deep: true, immediate: true })
</script>

<style scoped>
.file-uploader {
  width: 100%;
}

.drop-area {
  border: 2px dashed #d9d9d900;
  border-radius: 8px;
  padding: 20px 10px;
  text-align: center;
  cursor: pointer;
  transition: all 0.3s ease;
  background-color: #00000000;
}

.drop-area:hover {
  border-color: #2080f0;
  background-color: #5c5c5c6e;
}

.drop-area--active {
  border-color: #2080f0;
  background-color: #5c5c5c6e;
}
.file-item
{
    margin-top: 16px;
    /* background-color: #5556578e; */
}
</style>