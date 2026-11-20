<template>
  <div class="bg-logi-panel border border-logi-border/70 rounded-2xl p-2.5 flex items-center justify-between text-slate-100">
    <div class="flex items-center space-x-2">
      <span class="text-xs font-medium text-slate-400 pl-2">Active Profile:</span>
      <div class="flex items-center gap-1.5 overflow-x-auto">
        <button
          v-for="profile in store.profiles"
          :key="profile.id"
          @click="store.activeProfile = profile.id"
          class="px-3 py-1.5 rounded-xl text-xs font-medium transition-all duration-200 border"
          :class="
            store.activeProfile === profile.id
              ? 'bg-teal-500/15 border-teal-500/40 text-teal-300 shadow-sm'
              : 'bg-slate-800/60 border-transparent text-slate-400 hover:text-slate-200 hover:bg-slate-800'
          "
        >
          {{ profile.name }}
        </button>
      </div>
    </div>

    <div class="flex items-center gap-1.5 pr-1">
      <button
        class="p-1.5 rounded-lg bg-slate-800/80 hover:bg-slate-700 text-slate-300 hover:text-white transition-colors"
        title="Add Application Profile"
      >
        <Plus class="w-4 h-4" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { store } from '../stores/deviceStore';
import { Plus } from 'lucide-vue-next';
</script>
