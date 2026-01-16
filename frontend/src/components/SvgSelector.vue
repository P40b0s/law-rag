<template lang="pug">
.svg-uploader
  //- Отображение текущего SVG
  .svg-preview
    label-with-description.preview-header(name="Логотип" description="Логотип для текущего статуса, нажав на логотип можно загрузить новый, поддерживаются только файлы в формате .svg")
    .preview-content(@click="trigger_file_input")
      .svg-container(v-if="current_svg")
        div(v-html="current_svg")
      .upload-placeholder(v-else)
        | Нажмите для загрузки логотипа (только SVG)
      input(
        ref="file_input"
        type="file" 
        accept=".svg,image/svg+xml"
        style="display: none"
        @change="handle_upload")

    .preview-info
      n-text(depth="3") Размер: {{ svg_size }}
      n-button(size="small" @click="download_svg")
        template(#icon)
          n-icon
            DownloadOutline
        | Скачать
</template>

<script lang="ts" setup>
import { ref, computed, onMounted, watch } from 'vue'
import {
  NCard,
  NButton,
  NButtonGroup,
  NUpload,
  NUploadDragger,
  NText,
  NP,
  NIcon,
  NAlert,
  NSpace,
  NSpin,
  NTooltip,
  type UploadCustomRequestOptions
} from 'naive-ui'
import {
  CloudUploadOutline,
  DownloadOutline,
  TrashOutline,
  CloseOutline,
  InformationCircleOutline
} from '@vicons/ionicons5'
import { LabelWithDescription } from './label_with_description'
import { notify_service } from '@/services/notification_service'
import DOMPurify from 'dompurify'
// Types
interface SvgUploaderProps 
{
  svg?: string | null,
  max_size?: number
}

interface SvgUploaderEmits 
{
  (e: 'update:svg', svg: string | null): void
}
// Props & Emits
const props = withDefaults(defineProps<SvgUploaderProps>(), 
{
  svg: null,
  max_size: 1
})

const emit = defineEmits<SvgUploaderEmits>()

const current_svg = ref<string | null>(null)
const file_input = ref<HTMLInputElement|null>(null)
const svg_size = computed(() => 
{
  if (!current_svg.value) return '0 KB'
  const size = new Blob([current_svg.value]).size
  return size > 1024 ? `${(size / 1024).toFixed(1)} KB` : `${size} B`
})
const trigger_file_input = () => 
{
  file_input.value?.click()
}


const handle_upload = async (event: Event) => 
{
  const target = event.target as HTMLInputElement;
  const file = target.files?.item(0);
  if (!file) return;
  // Проверка типа файла
  if (file.type !== 'image/svg+xml' && !file.name.endsWith('.svg')) 
  {
    notify_service.notify_error("Файл должен быть в формате SVG", "");
    return;
  }

  // Проверка размера (2MB)
  if (file.size > props.max_size * 1024 * 1024) 
  {
    notify_service.notify_error(`Размер файла не должен превышать ${props.max_size}MB`, "");
    return;
  }

  // Чтение файла
  let svg_content = await read_file_as_text(file)

  // Валидация SVG
  if (!is_valid_svg(svg_content)) 
  {
    notify_service.notify_error('Файл не является валидным SVG', "")
    return;
  }
  current_svg.value = DOMPurify.sanitize(change_svg_size(svg_content));
  console.log(current_svg.value);
  emit('update:svg', svg_content)
}

//бывают svg без размеров, они корректно не отображаются, исправляем это
const change_svg_size = (content: string) =>
{
  // Проверяем, есть ли уже width/height
      const hasWidth = /width=/.test(content)
      const hasHeight = /height=/.test(content)
      
      if (!hasWidth || !hasHeight) 
      {
        // Добавляем только отсутствующие атрибуты
        if (!hasWidth && !hasHeight) {
          content = content.replace(
            /<svg([^>]*)>/,
            '<svg$1 width="200px" height="200px">'
          )
        } 
        else if (!hasWidth) 
        {
          content = content.replace(
            /<svg([^>]*)>/,
            '<svg$1 width="200px">'
          )
        } 
        else if (!hasHeight) 
        {
          content = content.replace(
            /<svg([^>]*)>/,
            '<svg$1 height="200px">'
          )
        }
      }
      return content;
}


const read_file_as_text = (file: File): Promise<string> => 
{
  return new Promise((resolve, reject) => 
  {
    const reader = new FileReader()
    reader.onload = (e) => resolve(e.target?.result as string)
    reader.onerror = () => reject(new Error('Ошибка чтения файла'))
    reader.readAsText(file)
  })
}

const is_valid_svg = (content: string): boolean => content.includes('</svg>');

const download_svg = () => 
{
  if (!current_svg.value) return

  const blob = new Blob([current_svg.value], { type: 'image/svg+xml' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = 'image.svg'
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}


const clear_svg = () => 
{
  current_svg.value = null
  emit('update:svg', null)
}

// Watch for prop changes
watch(() => props.svg, (new_svg) => 
{
  current_svg.value = new_svg;

}, {immediate: true})


</script>

<style lang="scss" scoped>
$max-height: 200px;
$max-width: 200px;
$min-height: 200px;
$min-width: 200px;
.svg-uploader 
{
  display: flex;
  flex-direction: column;
  gap: 16px;
  max-width: $max-width;
  max-height: $max-height;
}

.svg-preview 
{
  display: flex;
  flex-direction: column;
  gap: 2px;
  max-width: inherit;
}

.preview-header 
{
  display: flex;
  justify-content: center;
}

.preview-content 
{
  border: 2px dashed #d9d9d9;
  border-radius: 8px;
  padding: 20px;
  display: flex;
  justify-content: center;
  align-items: center;
  background: #fafafa;
  min-height: $min-height;
  min-width: $min-width;
  cursor: pointer;
  transition: border-color 0.3s ease;


  &:hover 
  {
    border-color: #409eff;
  }
}

.upload-placeholder 
{
  color: #666;
  text-align: center;
}
.svg-container  
{
  width: 100%;
  max-width: $max-width;
  max-height: $max-height;
  margin: 0 auto;
}
.svg-container  :deep(div) 
{
  display: flex;
  justify-content: center;
  align-items: center;
  max-height: inherit;
  max-width: inherit;
}
.svg-container  :deep(svg) 
{
  width: 100%;
  height: auto;
  max-height: inherit;
  max-width: 100%;
}

.preview-info 
{
  text-align: center;
}

// Адаптивность
@media (max-width: 270px) 
{
  .preview-content 
  {
    padding: 12px;
    min-height: 150px;
  }
}

</style>