<template>
  <div class="bg-logi-panel border border-logi-border/70 rounded-2xl p-4 text-slate-100">
    <div class="flex items-center justify-between mb-3">
      <div>
        <h4 class="text-xs font-semibold text-slate-200">Pointer Speed (DPI)</h4>
        <p class="text-[11px] text-slate-400">Darkfield 8000 DPI Optical Sensor</p>
      </div>
      <div class="px-2.5 py-1 rounded-lg bg-teal-500/10 border border-teal-500/20 text-teal-400 font-mono font-bold text-xs">
        {{ currentDpi }} DPI
      </div>
    </div>

    <!-- Range Slider -->
    <div class="my-2">
      <input
        type="range"
        min="200"
        max="8000"
        step="50"
        v-model.number="currentDpi"
        class="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-teal-400"
      />
    </div>

    <!-- Quick Presets -->
    <div class="flex items-center justify-between text-[11px] text-slate-400 mt-2">
      <span class="font-mono">200</span>
      <div class="flex gap-1.5">
        <button
          v-for="preset in [800, 1200, 1600, 2400, 4000]"
          :key="preset"
          @click="currentDpi = preset"
          class="px-2 py-0.5 rounded border border-slate-700 bg-slate-800/80 hover:bg-slate-700 transition-colors"
          :class="{ 'border-teal-400 text-teal-300 font-medium': currentDpi === preset }"
        >
          {{ preset }}
        </button>
      </div>
      <span class="font-mono">8000</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { store } from '../stores/deviceStore';

const currentDpi = computed({
  get: () => {
    const mouse = store.devices.find((d) => d.type === 'mouse');
    return mouse?.dpi?.current || 1600;
  },
  set: (val: number) => {
    store.setDpi(val);
  },
});
</script>
