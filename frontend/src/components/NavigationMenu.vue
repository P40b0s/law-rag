<template lang="pug">
n-space(vertical)
    n-layout.main(has-sider)
        n-layout-sider.menu-sider(
            v-if="menu_visible"
            bordered
            collapse-mode="width"
            :collapsed-width="collapsed_width"
            :width="240"
            :collapsed="collapsed"
            @collapse="collapsed = true"
            @expand="collapsed = false"
        )
            .menu
                n-menu.upper-menu(
                v-model:value="activeKey"
                :collapsed="collapsed"
                :collapsed-width="64"
                :collapsed-icon-size="32"
                :options="upperMenuOptions"
                )
                n-menu.down-menu(
                v-model:value="activeKey"
                :collapsed="collapsed"
                :collapsed-width="64"
                :collapsed-icon-size="45"
                :options="downMenuOptions"
                )
        n-layout-content.content
            router-view(v-slot="{ Component }" :key="route.fullPath")
                transition(name="fade" mode="out-in")
                    component(:is="Component")
</template>
    
<script lang="ts">
import type { MenuOption } from 'naive-ui'
import { NMenu, NIcon, NLayout, NSpace, NLayoutSider, NButton, NLayoutContent } from 'naive-ui'
import type { Component, ComputedRef, CSSProperties, Ref } from 'vue'
import { computed, defineComponent, h, onMounted, reactive, ref, watch } from 'vue'
import { RouterView, useRoute } from "vue-router";
import router from '@/router'
import Login from '@views/Login.vue'
import useVisible from '@composables/useVisible'
import { routes } from '@services/routes'
import { type Role } from '@/types/user_role';
import { type Permission } from '@/types/permission';

</script>

<script lang="ts" setup>
const route = useRoute()
const {visible} = useVisible()
const activeKey = ref<string | null>(router.currentRoute.value.name as string);
const collapsed = ref(true);
const collapsed_width = ref(64);

//for correct highlighting selected menu
watch(router.currentRoute, (r) =>
{
    activeKey.value = r.name as string;
})
const menu_visible = visible(['User', 'Administrator'])
const upperMenuOptions: ComputedRef<MenuOption[]> = computed(() => Array.from(routes.values())
.filter(f=>f.menu && f.menu.placement == 'up').map(route => ({
    show: route.visible.value,
    label: () => route.render_menu_label(),
    key: route.get_name(),
    icon: route.render_menu_icon(),
})))
const downMenuOptions: ComputedRef<MenuOption[]> = computed(() => Array.from(routes.values())
.filter(f=>f.menu && f.menu.placement == 'down').map(route => ({
    show: route.visible.value,
    label: () => route.render_menu_label(),
    key: route.get_name(),
    icon: route.render_menu_icon(),
})))
</script>
    
<style lang="scss" scoped>
.main
{
    height: 100vh;
}
.menu-sider
{
    background-color: var(--background-accent);

}
.n-menu-item-content .n-menu-item-content--selected
{
    transition: background-color .0s transparent;
    ::before
    {
        transition: background-color .0s transparent;
    }
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
.content 
{
    flex: 1;
    overflow: hidden;
    overflow-y: hidden;
    overflow-x: hidden;
    box-sizing: border-box;
    margin-left: 10px;
    margin-top: 5px;
}
.menu
{
    display: flex;
    flex-direction: column;
    height: 97vh;
}

.upper-menu
{
    flex-grow: 10;
}

</style>