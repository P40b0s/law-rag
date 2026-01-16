<template lang="pug">
.scroll
  component(v-if="node" :is="render_node")
</template>

<script setup lang="ts">
import { http_sevice } from '@/services/http_service/http_service';
import { type DynamicVNode } from '@/types/dynamic_vnode';
import { computed, h, onMounted, ref, type VNode } from 'vue';
import '@styles/systema.scss';
const props = defineProps<{
  doc_id: number;
}>();
const node = ref<DynamicVNode>();
onMounted(async () =>
{
  let n = await http_sevice.get_document_text(props.doc_id);
  if(n)
    node.value = n;
})

const render_node = computed(() => render(node.value))

const render = (vnode: DynamicVNode|undefined|null): VNode => 
{
  if(vnode)
  {
    const child = (vnode.children && Array.isArray(vnode.children))
        ? vnode.children.map(child => typeof child === 'object' ? render(child) : child)
        : typeof vnode.children === 'object' ? render(vnode.children) : vnode.children;

    return h(
      vnode.tag,
      vnode.attrs || {},
      child
    );
  }
  else
  {
    return h('div', [])
  }
};

</script>
<style lang="scss">
.scroll
{
	display: block;
  overflow-y: auto;
	scroll-behavior: smooth;
  height: 100vh;
	max-height: calc(100vh - 95px);
}
.viewer
{
	height: 100vh;
	max-height: calc(100vh - 65px);
}
</style>