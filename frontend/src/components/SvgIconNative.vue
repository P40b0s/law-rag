<template lang="pug">
.svg-container(v-if="props.svg")
  div(v-html="props.svg")
</template>
        
<script lang="ts">
import { computed, h, ref } from 'vue'
</script>

<script lang="ts" setup>
interface Props 
{
    svg: string,
    size?: number,
}
const props = withDefaults(defineProps<Props>(),
{
    size: 50
})

const handleError = () => 
{
  console.error('Failed to load SVG')
}

const width = ref(props.size + "px")
const height = ref(props.size + "px")
</script>
    
<style lang="scss" scoped>
// .svg
// {
// 	width: v-bind(width);
// 	height: v-bind(height);
// }
// .svg-wrapper {
//   position: relative;
//   border-radius: 50%;
//   padding: 0px; /* Отступ для свечения */
//   z-index: 1;
//   opacity: 1;
//   width: v-bind(width);
// 	height: v-bind(height);
  
//   &::before {
//     content: '';
//     position: absolute;
//     top: v-bind(highlight_size);
//     left: v-bind(highlight_size);
//     right: v-bind(highlight_size);
//     bottom: v-bind(bottom_highlight_size);
//     border-radius: 50%;
//     background: transparent;
//     z-index: -1;
//     transition: all 0.3s ease;

//   }

//   &:hover::before {
//     box-shadow: 
//       0 0 10px 3px rgba(100, 200, 255, 0.7), /* Голубое свечение */
//       0 0 20px 5px rgba(100, 200, 255, 0.4); /* Рассеянное свечение */
//   }
// }

/* Анимация пульсации (опционально) */
@keyframes pulse {
  0% { box-shadow: 0 0 5px 2px rgba(100, 200, 255, 0.5); }
  50% { box-shadow: 0 0 15px 5px rgba(100, 200, 255, 0.8); }
  100% { box-shadow: 0 0 5px 2px rgba(100, 200, 255, 0.5); }
}

.svg-wrapper.active::before {
  animation: pulse 2s infinite;
}
.svg-container  
{
  background: transparent;
  border-radius: 50%;
  width: 100%;
  max-width: v-bind(width);
  max-height: v-bind(height);
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
</style>