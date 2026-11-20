<template>
  <div class="w-full max-w-md mx-auto bg-logi-panel border border-logi-border/70 rounded-2xl shadow-2xl p-5 text-slate-100 backdrop-blur-md">
    <!-- Header -->
    <div class="flex items-center justify-between pb-4 border-b border-logi-border/50">
      <div class="flex items-center space-x-2.5">
        <div class="w-8 h-8 rounded-lg bg-teal-500/10 border border-teal-500/20 flex items-center justify-center text-teal-400">
          <Zap class="w-4 h-4" />
        </div>
        <div>
          <h2 class="font-semibold text-sm tracking-wide flex items-center gap-1.5">
            LogiBatteryBar
            <span class="text-[10px] uppercase font-mono px-1.5 py-0.5 rounded bg-teal-500/10 text-teal-400 border border-teal-500/20">Linux</span>
          </h2>
          <p class="text-[11px] text-slate-400">Logi Bolt Wireless Monitor</p>
        </div>
      </div>
      <div class="flex items-center space-x-1.5">
        <button
          @click="handleRefresh"
          class="p-2 rounded-lg bg-slate-800/80 hover:bg-slate-700/80 text-slate-300 hover:text-white transition-all duration-200"
          :class="{ 'animate-spin': isRefreshing }"
          title="Refresh Battery Telemetry"
        >
          <RefreshCw class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>

    <!-- Device List Cards -->
    <div class="mt-4 space-y-3">
      <div
        v-for="device in store.devices"
        :key="device.id"
        class="bg-logi-card/80 border border-logi-border/60 hover:border-slate-500/50 rounded-xl p-3.5 transition-all duration-200"
      >
        <div class="flex items-start justify-between">
          <div class="flex items-center space-x-3">
            <div class="w-9 h-9 rounded-xl bg-slate-800 flex items-center justify-center text-slate-300">
              <Mouse v-if="device.type === 'mouse'" class="w-5 h-5 text-teal-400" />
              <Keyboard v-else class="w-5 h-5 text-indigo-400" />
            </div>
            <div>
              <div class="flex items-center gap-2">
                <h3 class="font-medium text-sm text-slate-100">{{ device.name }}</h3>
                <span class="text-[10px] font-mono px-1.5 py-0.2 rounded bg-slate-800 text-slate-400 border border-slate-700">
                  Slot {{ device.slot }}
                </span>
              </div>
              <div class="flex items-center gap-2 mt-1">
                <!-- Status Badge -->
                <span
                  v-if="device.battery.charging"
                  class="inline-flex items-center gap-1 text-[11px] px-2 py-0.5 rounded-full font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20"
                >
                  <Zap class="w-3 h-3 animate-pulse" /> Charging via USB
                </span>
                <span
                  v-else-if="device.battery.status === 'standby'"
                  class="inline-flex items-center gap-1 text-[11px] px-2 py-0.5 rounded-full font-medium bg-amber-500/10 text-amber-400 border border-amber-500/20"
                >
                  <Moon class="w-3 h-3" /> Standby (Idle)
                </span>
                <span
                  v-else
                  class="inline-flex items-center gap-1 text-[11px] px-2 py-0.5 rounded-full font-medium bg-teal-500/10 text-teal-300 border border-teal-500/20"
                >
                  <Check class="w-3 h-3" /> Connected
                </span>
              </div>
            </div>
          </div>

          <!-- Percentage Indicator -->
          <div class="text-right">
            <div class="text-xl font-bold tracking-tight" :class="getBatteryTextColor(device.battery.percentage)">
              {{ device.battery.percentage }}%
            </div>
            <div class="text-[10px] text-slate-500 mt-0.5">
              {{ device.updatedAt }}
            </div>
          </div>
        </div>

        <!-- Progress Bar -->
        <div class="mt-3">
          <div class="w-full bg-slate-800 rounded-full h-2 overflow-hidden border border-slate-700/50">
            <div
              class="h-full rounded-full transition-all duration-500"
              :class="getBatteryBarColor(device.battery.percentage)"
              :style="{ width: `${device.battery.percentage}%` }"
            ></div>
          </div>
        </div>
      </div>
    </div>

    <!-- Footer Controls -->
    <div class="mt-5 pt-3.5 border-t border-logi-border/40 flex items-center justify-between text-xs text-slate-400">
      <label class="flex items-center gap-2 cursor-pointer hover:text-slate-200 transition-colors">
        <input
          type="checkbox"
          v-model="store.startAtLogin"
          class="rounded bg-slate-800 border-slate-700 text-teal-500 focus:ring-0 w-3.5 h-3.5"
        />
        <span>Start at Login</span>
      </label>

      <button
        @click="store.activeTab = 'device_options'"
        class="text-teal-400 hover:text-teal-300 font-medium flex items-center gap-1 transition-colors"
      >
        <span>Open Options</span>
        <ChevronRight class="w-3.5 h-3.5" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { store } from '../stores/deviceStore';
import {
  Zap,
  RefreshCw,
  Mouse,
  Keyboard,
  Moon,
  Check,
  ChevronRight,
} from 'lucide-vue-next';

const isRefreshing = ref(false);

function handleRefresh() {
  isRefreshing.value = true;
  store.refreshBattery();
  setTimeout(() => {
    isRefreshing.value = false;
  }, 600);
}

function getBatteryTextColor(pct: number) {
  if (pct > 30) return 'text-emerald-400';
  if (pct >= 15) return 'text-amber-400';
  return 'text-rose-400';
}

function getBatteryBarColor(pct: number) {
  if (pct > 30) return 'bg-emerald-500';
  if (pct >= 15) return 'bg-amber-500';
  return 'bg-rose-500';
}
</script>
