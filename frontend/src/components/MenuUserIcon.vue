<template lang="pug">
user-icon(v-if="id" :user_id="id.id" :size="39" :visible_if_not_found="true")
</template>
        
<script lang="ts">
import useUser from '@composables/useUser'
import UserIcon from './UserIcon.vue'
import { onMounted, ref, watch } from 'vue';
</script>
<script lang="ts" setup>

const {get_user} = useUser();
const id = get_user();

</script>
<style lang="scss" scoped>
</style>