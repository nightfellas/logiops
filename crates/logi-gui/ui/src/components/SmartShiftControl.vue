<template>
  <div class="bg-logi-panel border border-logi-border/70 rounded-2xl p-4 text-slate-100">
    <div class="flex items-center justify-between mb-3">
      <div>
        <h4 class="text-xs font-semibold text-slate-200">SmartShift Wheel</h4>
        <p class="text-[11px] text-slate-400">Automatic ratchet & free-spin engagement</p>
      </div>
      <label class="relative inline-flex items-center cursor-pointer">
        <input
          type="checkbox"
          v-model="enabled"
          class="sr-only peer"
        />
        <div class="w-9 h-5 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-teal-500"></div>
      </label>
    </div>

    <!-- Threshold Sensitivity Slider -->
    <div v-if="enabled" class="space-y-2 mt-3 pt-3 border-t border-logi-border/50">
      <div class="flex items-center justify-between text-[11px] text-slate-400">
        <span>SmartShift Sensitivity (Torque)</span>
        <span class="font-mono text-slate-200">{{ threshold }}</span>
      </div>
      <input
        type="range"
        min="1"
        max="100"
        v-model.number="threshold"
        class="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-teal-400"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { store } from '../stores/deviceStore';

const mouse = computed(() => store.devices.find((d) => d.type === 'mouse'));

const enabled = computed({
  get: () => mouse.value?.smartshift?.enabled ?? true,
  set: (val: boolean) => {
    if (mouse.value && mouse.value.smartshift) {
      store.setSmartShift(val, mouse.value.smartshift.threshold);
    }
  },
});

const threshold = computed({
  get: () => mouse.value?.smartshift?.threshold ?? 32,
  set: (val: number) => {
    if (mouse.value && mouse.value.smartshift) {
      store.setSmartShift(mouse.value.smartshift.enabled, val);
    }
  },
});
</script>
