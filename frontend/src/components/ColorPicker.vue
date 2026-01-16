<template lang="pug">
n-tooltip
  template(#trigger)
    .color-picker
      .color-preview(
        :style="{ backgroundColor: selectedColor }"
        @click="showPicker = !showPicker")
        .picker-popover(v-if="showPicker" @click.stop)
          n-card
            .color-grid
              .color-square(
                v-for="color in colorPalette"
                :key="color"
                :style="{ backgroundColor: color }"
                @click="selectColor(color)"
                :class="{ active: selectedColor === color }"
              )
            .custom-color
              input(
                type="color"
                v-model="customColor"
                @change="selectCustomColor"
              )
              span Другой цвет
            template(#action)
              .action
                n-button(@click="cancelPicker" type="error" size="small") Отмена
                n-button(@click="applyColor" type="success" size="small") Применить
  div Выберите цвет для данного статуса
            
</template>
<script setup lang="ts">
import { ref, watch } from 'vue'
import { NCard, NButton, NTooltip } from 'naive-ui';
interface Props {
  value?: string
}

const props = withDefaults(defineProps<Props>(), {
  value: '#000000'
})

const emit = defineEmits<{
  'update:value': [value: string]
}>()

const showPicker = ref(false)
const selectedColor = ref(props.value)
const customColor = ref('#000000')

// Палитра предустановленных цветов
const colorPalette = ref([
  '#FF0000', '#00FF00', '#0000FF', '#FFFF00', '#FF00FF', '#00FFFF',
  '#000000', '#FFFFFF', '#808080', '#FFA500', '#800080', '#008000',
  '#800000', '#008080', '#000080', '#89DAD3', '#FF6B6B', '#4ECDC4'
])

const selectColor = (color: string) => {
  selectedColor.value = color
  customColor.value = color
}

const selectCustomColor = () => {
  selectedColor.value = customColor.value
}

const applyColor = () => {
  emit('update:value', selectedColor.value)
  showPicker.value = false
}

const cancelPicker = () => {
  selectedColor.value = props.value
  customColor.value = props.value
  showPicker.value = false
}

// Следим за изменением значения извне
watch(() => props.value, (newValue) => {
  selectedColor.value = newValue
  customColor.value = newValue
})

// Закрываем пикер при клике вне компонента
const handleClickOutside = (event: Event) => {
  const target = event.target as HTMLElement
  if (!target.closest('.color-picker')) {
    showPicker.value = false
  }
}

// Добавляем обработчик клика вне компонента
if (typeof window !== 'undefined') {
  window.addEventListener('click', handleClickOutside)
}
</script>
<style scoped>
.color-picker {
  position: relative;
  display: inline-block;
}

.color-preview {
  width: 40px;
  height: 40px;
  border: 2px solid #ddd;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}
.action
{
  display: flex;
  flex-direction: row;
  justify-content: space-between;
}

.color-preview:hover {
  border-color: #999;
}

.picker-popover {
  position: absolute;
  top: 50px;
  left: 0;
  background: rgba(255, 255, 255, 0);
  border: 1px solid #dddddd00;
  border-radius: 8px;
  padding: 1px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  z-index: 1000;
  min-width: 250px;
}

.color-grid {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  gap: 4px;
  margin-bottom: 12px;
}

.color-square {
  width: 24px;
  height: 24px;
  border: 1px solid #ddd;
  border-radius: 3px;
  cursor: pointer;
  transition: transform 0.1s;
}

.color-square:hover {
  transform: scale(1.1);
}

.color-square.active {
  border: 2px solid #333;
  transform: scale(1.1);
}

.custom-color {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.custom-color input[type="color"] {
  width: 40px;
  height: 30px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.actions button {
  padding: 6px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: white;
  cursor: pointer;
  transition: background-color 0.2s;
}

.actions button:hover {
  background-color: #f5f5f5;
}

.actions button:first-child {
  background-color: #007bff;
  color: white;
  border-color: #007bff;
}

.actions button:first-child:hover {
  background-color: #0056b3;
}
</style>