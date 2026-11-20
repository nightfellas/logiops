<template>
  <div class="relative w-full max-w-xl mx-auto flex items-center justify-center p-6 bg-logi-card/40 border border-logi-border/40 rounded-3xl backdrop-blur-sm">
    <!-- SVG Diagram for Logitech MX Master 3S -->
    <svg viewBox="0 0 500 650" class="w-full max-h-[500px] drop-shadow-2xl">
      <defs>
        <linearGradient id="bodyGrad" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="#2a2e38" />
          <stop offset="50%" stop-color="#1f2229" />
          <stop offset="100%" stop-color="#15171c" />
        </linearGradient>
        <linearGradient id="metalGrad" x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%" stop-color="#8e97a4" />
          <stop offset="50%" stop-color="#4a505b" />
          <stop offset="100%" stop-color="#2a2e37" />
        </linearGradient>
        <filter id="glow">
          <feGaussianBlur stdDeviation="3" result="blur" />
          <feComposite in="SourceGraphic" in2="blur" operator="over" />
        </filter>
      </defs>

      <!-- Ergonomic Thumb Rest Wing -->
      <path
        d="M 120 320 C 80 340, 50 400, 70 480 C 85 530, 140 550, 190 560 Z"
        fill="#22252d"
        stroke="#383c48"
        stroke-width="2"
      />

      <!-- Main Mouse Body Silhouette -->
      <path
        d="M 250 80
           C 340 80, 400 160, 410 280
           C 420 400, 390 520, 330 580
           C 270 630, 190 610, 150 560
           C 110 500, 120 350, 140 260
           C 160 170, 190 80, 250 80 Z"
        fill="url(#bodyGrad)"
        stroke="#383c48"
        stroke-width="2"
      />

      <!-- Split Left & Right Click Plate -->
      <path
        d="M 250 82 L 250 250 M 165 240 C 210 260, 290 260, 385 240"
        fill="none"
        stroke="#1a1c22"
        stroke-width="3"
      />

      <!-- Magspeed Scroll Wheel (Center) -->
      <rect
        x="235"
        y="130"
        width="30"
        height="85"
        rx="10"
        fill="url(#metalGrad)"
        stroke="#1a1c23"
        stroke-width="2"
      />
      <!-- Wheel Ridges -->
      <line x1="235" y1="150" x2="265" y2="150" stroke="#121316" stroke-width="2" />
      <line x1="235" y1="165" x2="265" y2="165" stroke="#121316" stroke-width="2" />
      <line x1="235" y1="180" x2="265" y2="180" stroke="#121316" stroke-width="2" />
      <line x1="235" y1="195" x2="265" y2="195" stroke="#121316" stroke-width="2" />

      <!-- Mode Shift / DPI Button -->
      <rect
        x="241"
        y="235"
        width="18"
        height="26"
        rx="6"
        fill="#323640"
        stroke="#4a505f"
        stroke-width="1.5"
      />

      <!-- Thumb Wheel (Horizontal Scroll) -->
      <rect
        x="95"
        y="300"
        width="22"
        height="55"
        rx="6"
        fill="url(#metalGrad)"
        stroke="#1a1c23"
        stroke-width="1.5"
      />
      <line x1="95" y1="315" x2="117" y2="315" stroke="#121316" stroke-width="1.5" />
      <line x1="95" y1="330" x2="117" y2="330" stroke="#121316" stroke-width="1.5" />
      <line x1="95" y1="345" x2="117" y2="345" stroke="#121316" stroke-width="1.5" />

      <!-- Thumb Forward & Back Buttons -->
      <path
        d="M 125 365 L 140 365 L 142 390 L 127 390 Z"
        fill="#323640"
        stroke="#4a505f"
        stroke-width="1"
      />
      <path
        d="M 128 395 L 143 395 L 145 420 L 130 420 Z"
        fill="#323640"
        stroke="#4a505f"
        stroke-width="1"
      />

      <!-- Gesture Pad Indicator Lines on Wing -->
      <path
        d="M 90 440 C 95 470, 115 490, 140 500"
        fill="none"
        stroke="#383d49"
        stroke-dasharray="3,3"
        stroke-width="2"
      />

      <!-- INTERACTIVE CLICK PINS -->

      <!-- 1. Middle Wheel Pin -->
      <g @click="selectButton('middle_button')" class="cursor-pointer group">
        <circle cx="250" cy="172" r="14" fill="#00d2c4" fill-opacity="0.2" class="group-hover:scale-125 transition-transform" />
        <circle cx="250" cy="172" r="7" fill="#00d2c4" filter="url(#glow)" />
      </g>

      <!-- 2. Mode Shift Pin -->
      <g @click="selectButton('dpi_switch')" class="cursor-pointer group">
        <circle cx="250" cy="248" r="14" fill="#00d2c4" fill-opacity="0.2" class="group-hover:scale-125 transition-transform" />
        <circle cx="250" cy="248" r="7" fill="#00d2c4" filter="url(#glow)" />
      </g>

      <!-- 3. Thumb Wheel Pin -->
      <g @click="selectButton('thumb_wheel')" class="cursor-pointer group">
        <circle cx="106" cy="327" r="14" fill="#00d2c4" fill-opacity="0.2" class="group-hover:scale-125 transition-transform" />
        <circle cx="106" cy="327" r="7" fill="#00d2c4" filter="url(#glow)" />
      </g>

      <!-- 4. Thumb Forward Pin -->
      <g @click="selectButton('thumb_forward')" class="cursor-pointer group">
        <circle cx="134" cy="377" r="12" fill="#00d2c4" fill-opacity="0.2" class="group-hover:scale-125 transition-transform" />
        <circle cx="134" cy="377" r="6" fill="#00d2c4" filter="url(#glow)" />
      </g>

      <!-- 5. Thumb Back Pin -->
      <g @click="selectButton('thumb_back')" class="cursor-pointer group">
        <circle cx="137" cy="407" r="12" fill="#00d2c4" fill-opacity="0.2" class="group-hover:scale-125 transition-transform" />
        <circle cx="137" cy="407" r="6" fill="#00d2c4" filter="url(#glow)" />
      </g>

      <!-- 6. Thumb Gesture Pad Pin -->
      <g @click="selectButton('gesture_button')" class="cursor-pointer group">
        <circle cx="115" cy="470" r="16" fill="#00d2c4" fill-opacity="0.25" class="animate-pulse" />
        <circle cx="115" cy="470" r="8" fill="#00d2c4" filter="url(#glow)" />
      </g>
    </svg>

    <!-- Floating Pin Labels -->
    <div
      @click="selectButton('gesture_button')"
      class="absolute left-6 bottom-16 bg-slate-900/90 border border-teal-500/40 text-teal-300 text-xs px-3 py-1.5 rounded-xl shadow-lg cursor-pointer hover:border-teal-400 transition-all flex items-center gap-1.5"
    >
      <span class="w-2 h-2 rounded-full bg-teal-400 animate-ping"></span>
      <span>Gesture Button</span>
    </div>

    <div
      @click="selectButton('thumb_wheel')"
      class="absolute left-4 top-40 bg-slate-900/90 border border-teal-500/40 text-teal-300 text-xs px-3 py-1.5 rounded-xl shadow-lg cursor-pointer hover:border-teal-400 transition-all"
    >
      <span>Thumb Wheel</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { store } from '../stores/deviceStore';

function selectButton(id: string) {
  store.selectedButton = store.buttonMappings[id] || null;
}
</script>
