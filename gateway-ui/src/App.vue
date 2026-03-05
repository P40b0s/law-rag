<script setup lang="ts">
import { darkTheme, NConfigProvider, NMessageProvider, ruRU, dateRuRU } from 'naive-ui'
</script>

<template lang="pug">
n-config-provider(:theme="darkTheme" :locale="ruRU" :date-locale="dateRuRU")
    n-message-provider
        router-view
</template>
