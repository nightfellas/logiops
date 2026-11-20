<template>
  <div v-if="store.selectedButton" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm">
    <div class="w-full max-w-lg bg-logi-panel border border-logi-border rounded-2xl shadow-2xl p-6 text-slate-100 animate-in fade-in zoom-in-95 duration-150">
      <!-- Modal Header -->
      <div class="flex items-center justify-between pb-4 border-b border-logi-border/60">
        <div>
          <h3 class="text-base font-semibold text-slate-100 flex items-center gap-2">
            <span>{{ store.selectedButton.label }}</span>
            <span class="text-[11px] font-mono px-2 py-0.5 rounded-full bg-teal-500/10 text-teal-400 border border-teal-500/20">
              {{ store.selectedButton.id }}
            </span>
          </h3>
          <p class="text-xs text-slate-400 mt-0.5">Select an action for the active profile</p>
        </div>
        <button
          @click="store.selectedButton = null"
          class="p-1.5 rounded-lg text-slate-400 hover:text-slate-200 hover:bg-slate-800 transition-colors"
        >
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- Action Presets -->
      <div class="mt-4 space-y-4">
        <!-- Preset 1: Gestures -->
        <div
          v-if="store.selectedButton.id === 'gesture_button'"
          class="p-4 rounded-xl border border-teal-500/30 bg-teal-500/5"
        >
          <h4 class="text-xs font-semibold text-teal-300 uppercase tracking-wider mb-2">Directional Gesture Setup</h4>
          <div class="grid grid-cols-2 gap-2 text-xs">
            <div class="bg-slate-900/80 p-2.5 rounded-lg border border-slate-700">
              <span class="text-slate-400 font-mono">Tap Click:</span>
              <span class="block font-medium text-slate-200 mt-0.5">Overview / Mission Control</span>
            </div>
            <div class="bg-slate-900/80 p-2.5 rounded-lg border border-slate-700">
              <span class="text-slate-400 font-mono">Swipe Left:</span>
              <span class="block font-medium text-slate-200 mt-0.5">Previous Workspace</span>
            </div>
            <div class="bg-slate-900/80 p-2.5 rounded-lg border border-slate-700">
              <span class="text-slate-400 font-mono">Swipe Right:</span>
              <span class="block font-medium text-slate-200 mt-0.5">Next Workspace</span>
            </div>
            <div class="bg-slate-900/80 p-2.5 rounded-lg border border-slate-700">
              <span class="text-slate-400 font-mono">Swipe Up:</span>
              <span class="block font-medium text-slate-200 mt-0.5">Maximize Window</span>
            </div>
          </div>
        </div>

        <!-- Presets Selection List -->
        <div>
          <label class="block text-xs font-medium text-slate-300 mb-2">Popular Actions</label>
          <div class="space-y-1.5 max-h-60 overflow-y-auto pr-1">
            <button
              v-for="act in availableActions"
              :key="act.name"
              @click="applyAction(act.name)"
              class="w-full text-left px-3.5 py-2.5 rounded-xl border border-logi-border/60 hover:border-teal-500/40 hover:bg-slate-800/80 transition-all flex items-center justify-between text-xs"
              :class="{ 'border-teal-500/60 bg-teal-500/10 text-teal-300': store.selectedButton.actionName === act.name }"
            >
              <div>
                <span class="font-medium text-slate-200">{{ act.name }}</span>
                <span class="block text-[11px] text-slate-400 mt-0.5">{{ act.desc }}</span>
              </div>
              <Check v-if="store.selectedButton.actionName === act.name" class="w-4 h-4 text-teal-400" />
            </button>
          </div>
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="mt-6 pt-4 border-t border-logi-border/60 flex items-center justify-end space-x-2">
        <button
          @click="store.selectedButton = null"
          class="px-4 py-2 rounded-xl text-xs font-medium bg-slate-800 text-slate-300 hover:bg-slate-700 transition-colors"
        >
          Cancel
        </button>
        <button
          @click="saveAndClose"
          class="px-5 py-2 rounded-xl text-xs font-medium bg-teal-500 hover:bg-teal-400 text-slate-950 transition-colors"
        >
          Save Action
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { store } from '../stores/deviceStore';
import { X, Check } from 'lucide-vue-next';

const availableActions = [
  { name: 'Mission Control / Overview (Swipe for Workspaces)', desc: 'Window switcher & virtual desktop navigation' },
  { name: 'Next Tab (Ctrl + Tab)', desc: 'Switch to next browser / editor tab' },
  { name: 'Previous Tab (Ctrl + Shift + Tab)', desc: 'Switch to previous browser / editor tab' },
  { name: 'Close Window / Tab (Ctrl + W)', desc: 'Close current active tab' },
  { name: 'Show Desktop', desc: 'Minimize all open windows' },
  { name: 'Volume Up', desc: 'Increase master audio volume' },
  { name: 'Volume Down', desc: 'Decrease master audio volume' },
  { name: 'Play / Pause Media', desc: 'Toggle media playback' },
  { name: 'Native Middle Click', desc: 'Standard middle mouse button' },
];

function applyAction(actionName: string) {
  if (store.selectedButton) {
    store.selectedButton.actionName = actionName;
  }
}

function saveAndClose() {
  if (store.selectedButton) {
    // Send updated button mapping to daemon API
    fetch('/api/save_action', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        button_id: store.selectedButton.id,
        action_name: store.selectedButton.actionName,
      }),
    }).catch((e) => console.error('Failed to sync action:', e));
  }
  store.selectedButton = null;
}
</script>
