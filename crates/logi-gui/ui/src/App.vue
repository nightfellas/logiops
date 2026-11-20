<template>
  <div class="min-h-screen bg-logi-dark text-slate-100 flex flex-col">
    <!-- Top Navigation Bar -->
    <header class="border-b border-logi-border/70 bg-logi-panel/80 backdrop-blur-md sticky top-0 z-40 px-6 py-3.5 flex items-center justify-between">
      <div class="flex items-center space-x-3">
        <div class="w-8 h-8 rounded-xl bg-teal-500/15 border border-teal-500/30 flex items-center justify-center text-teal-400">
          <Layers class="w-4 h-4" />
        </div>
        <div>
          <h1 class="text-sm font-semibold tracking-wide flex items-center gap-2">
            LogiOptions-Linux
            <span class="text-[10px] font-mono font-medium px-2 py-0.5 rounded-full bg-slate-800 text-teal-400 border border-slate-700">
              v0.1.0
            </span>
          </h1>
        </div>
      </div>

      <!-- Center Tabs Switcher -->
      <div class="flex items-center p-1 bg-slate-900/90 border border-logi-border/60 rounded-xl space-x-1">
        <button
          @click="store.activeTab = 'battery_bar'"
          class="flex items-center gap-2 px-3.5 py-1.5 rounded-lg text-xs font-medium transition-all"
          :class="
            store.activeTab === 'battery_bar'
              ? 'bg-teal-500/20 text-teal-300 shadow-sm border border-teal-500/30'
              : 'text-slate-400 hover:text-slate-200'
          "
        >
          <Zap class="w-3.5 h-3.5" />
          <span>LogiBatteryBar</span>
        </button>

        <button
          @click="store.activeTab = 'device_options'"
          class="flex items-center gap-2 px-3.5 py-1.5 rounded-lg text-xs font-medium transition-all"
          :class="
            store.activeTab === 'device_options'
              ? 'bg-teal-500/20 text-teal-300 shadow-sm border border-teal-500/30'
              : 'text-slate-400 hover:text-slate-200'
          "
        >
          <Mouse class="w-3.5 h-3.5" />
          <span>MX Master 3S</span>
        </button>
      </div>

      <!-- Daemon Connection Status -->
      <div class="flex items-center space-x-2 text-xs">
        <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
        <span class="text-slate-400 text-[11px] font-mono">Daemon Online</span>
      </div>
    </header>

    <!-- Main Content Body -->
    <main class="flex-1 p-6 flex flex-col items-center justify-center">
      <!-- Tab 1: LogiBatteryBar Popover -->
      <div v-if="store.activeTab === 'battery_bar'" class="w-full max-w-md animate-in fade-in duration-200">
        <BatteryPopover />
      </div>

      <!-- Tab 2: Full MX Master 3S Options -->
      <div v-else class="w-full max-w-5xl space-y-6 animate-in fade-in duration-200">
        <!-- Profile Selection Bar -->
        <ProfileBar />

        <!-- Two Column Main Layout -->
        <div class="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
          <!-- Left Column: Interactive 2D SVG Mouse Diagram -->
          <div class="lg:col-span-7">
            <DeviceVisual />
          </div>

          <!-- Right Column: Settings & Mapping Controls -->
          <div class="lg:col-span-5 space-y-4">
            <!-- DPI Adjustment -->
            <DpiSlider />

            <!-- SmartShift Wheel Control -->
            <SmartShiftControl />

            <!-- Active Mappings Overview Card -->
            <div class="bg-logi-panel border border-logi-border/70 rounded-2xl p-4 text-slate-100">
              <h4 class="text-xs font-semibold text-slate-200 mb-2.5">Configured Buttons</h4>
              <div class="space-y-2 text-xs">
                <div
                  v-for="(btn, key) in store.buttonMappings"
                  :key="key"
                  @click="store.selectedButton = btn"
                  class="p-2.5 rounded-xl border border-slate-800 bg-slate-900/60 hover:border-teal-500/30 hover:bg-slate-800/80 cursor-pointer transition-all flex items-center justify-between"
                >
                  <div>
                    <span class="font-medium text-slate-300">{{ btn.label }}</span>
                    <span class="block text-[11px] text-teal-400/90 font-mono mt-0.5 truncate max-w-[200px]">
                      {{ btn.actionName }}
                    </span>
                  </div>
                  <ChevronRight class="w-3.5 h-3.5 text-slate-500" />
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </main>

    <!-- Button Action Modal -->
    <ButtonConfigModal />
  </div>
</template>

<script setup lang="ts">
import { store } from './stores/deviceStore';
import BatteryPopover from './components/BatteryPopover.vue';
import DeviceVisual from './components/DeviceVisual.vue';
import ButtonConfigModal from './components/ButtonConfigModal.vue';
import DpiSlider from './components/DpiSlider.vue';
import SmartShiftControl from './components/SmartShiftControl.vue';
import ProfileBar from './components/ProfileBar.vue';
import { Layers, Zap, Mouse, ChevronRight } from 'lucide-vue-next';
</script>
