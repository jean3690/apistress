<template>
  <div class="flex flex-col h-screen w-screen overflow-hidden">
    <ToolBar />
    <div class="flex-1 overflow-hidden flex">
      <MainLayout />
    </div>
    <StatusBar />
  </div>
</template>
<script setup lang="ts">
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts'
import ToolBar from './ToolBar.vue'
import StatusBar from './StatusBar.vue'
import MainLayout from './MainLayout.vue'

useKeyboardShortcuts()
</script>
