import { defineComponent, h, PropType } from 'vue'
import type { ToastOptions } from 'vue3-toastify'

export default defineComponent({
  name: 'Msg',
  props: {
    closeToast: Function as PropType<(e?: MouseEvent) => void>,
    toastProps: Object as PropType<ToastOptions>,
    title: {
      type: String,
      required: true
    },
    body: String as PropType<string | null>,
  },
  setup(props) 
  {
    return () => 
    {
      const children = [
        h('p', props.title)
      ]
      
      if (props.body) 
      {
        children.push(h('p', props.body))
      }
      
      return h('div', children)
    }
  }
})