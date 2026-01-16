<template lang="pug">
.svg-wrapper
	img.svg(
		:src="svg" 
		@error="handleError")
</template>
        
<script lang="ts">
import { computed, h, ref } from 'vue'
import { NIcon } from 'naive-ui'
</script>

<script lang="ts" setup>
interface Props 
{
    svg: string,
    size: number,
    //чем больше значение - тем меньше
    highlight_size?: number
}
const props = withDefaults(defineProps<Props>(),
{
    highlight_size: 3
})

const handleError = () => 
{
  console.error('Failed to load SVG')
}

const width = ref(props.size + "px")
const height = ref(props.size + "px")
const highlight_size = ref(props.highlight_size + "px")
const bottom_highlight_size = ref(props.highlight_size + 7 + "px")
</script>
    
<style lang="scss" scoped>
.svg
{
	width: v-bind(width);
	height: v-bind(height);
}
.svg-wrapper {
  position: relative;
  border-radius: 50%;
  padding: 0px; /* Отступ для свечения */
  z-index: 1;
  opacity: 1;
  width: v-bind(width);
	height: v-bind(height);
  
  &::before {
    content: '';
    position: absolute;
    top: v-bind(highlight_size);
    left: v-bind(highlight_size);
    right: v-bind(highlight_size);
    bottom: v-bind(bottom_highlight_size);
    border-radius: 50%;
    background: transparent;
    z-index: -1;
    transition: all 0.3s ease;

  }

  &:hover::before {
    box-shadow: 
      0 0 10px 3px rgba(100, 200, 255, 0.7), /* Голубое свечение */
      0 0 20px 5px rgba(100, 200, 255, 0.4); /* Рассеянное свечение */
  }
}

/* Анимация пульсации (опционально) */
@keyframes pulse {
  0% { box-shadow: 0 0 5px 2px rgba(100, 200, 255, 0.5); }
  50% { box-shadow: 0 0 15px 5px rgba(100, 200, 255, 0.8); }
  100% { box-shadow: 0 0 5px 2px rgba(100, 200, 255, 0.5); }
}

.svg-wrapper.active::before {
  animation: pulse 2s infinite;
}
// .svg-wrapper::before {
//   background: linear-gradient(45deg, #00ffff, #ff00ff);
//   opacity: 0;
//   transition: opacity 0.3s;
// }

// .svg-wrapper:hover::before {
//   opacity: 0.7;
//   box-shadow: 
//     0 0 10px 5px #00ffff,
//     0 0 20px 10px #ff00ff;
// }
</style>