import { reactive } from 'vue';

export interface DeviceInfo {
  id: string;
  name: string;
  type: 'mouse' | 'keyboard';
  slot: number;
  battery: {
    percentage: number;
    charging: boolean;
    critical: boolean;
    status: 'discharging' | 'charging' | 'full' | 'standby' | 'unknown';
  };
  dpi?: {
    current: number;
    min: number;
    max: number;
    step: number;
  };
  smartshift?: {
    enabled: boolean;
    threshold: number;
    mode: 'always_ratchet' | 'always_free_spin' | 'auto_disengage';
  };
  updatedAt: string;
}

export interface ButtonMapping {
  id: string;
  label: string;
  type: 'gesture' | 'key_combo' | 'native';
  actionName: string;
}

export const store = reactive({
  connected: true,
  startAtLogin: true,
  activeTab: 'battery_bar' as 'battery_bar' | 'device_options',
  activeProfile: 'global_default',
  profiles: [
    { id: 'global_default', name: 'Global (Default)', app: '*' },
    { id: 'chrome', name: 'Google Chrome', app: 'google-chrome' },
    { id: 'code', name: 'Visual Studio Code', app: 'code' },
    { id: 'davinci', name: 'DaVinci Resolve', app: 'davinci-resolve' },
  ],
  devices: [
    {
      id: 'bolt:slot_1',
      name: 'MX Master 3S',
      type: 'mouse',
      slot: 1,
      battery: {
        percentage: 75,
        charging: false,
        critical: false,
        status: 'discharging',
      },
      dpi: {
        current: 1600,
        min: 200,
        max: 8000,
        step: 50,
      },
      smartshift: {
        enabled: true,
        threshold: 32,
        mode: 'auto_disengage',
      },
      updatedAt: '19:10:42',
    },
    {
      id: 'bolt:slot_2',
      name: 'MX Mechanical Mini',
      type: 'keyboard',
      slot: 2,
      battery: {
        percentage: 80,
        charging: false,
        critical: false,
        status: 'standby',
      },
      updatedAt: '19:08:15',
    },
  ] as DeviceInfo[],
  buttonMappings: {
    gesture_button: {
      id: 'gesture_button',
      label: 'Thumb Gesture Button',
      type: 'gesture',
      actionName: 'Mission Control / Overview (Swipe for Workspaces)',
    },
    thumb_wheel: {
      id: 'thumb_wheel',
      label: 'Thumb Wheel',
      type: 'native',
      actionName: 'Horizontal Scroll / Timeline Scrub',
    },
    thumb_forward: {
      id: 'thumb_forward',
      label: 'Forward Button',
      type: 'key_combo',
      actionName: 'Next Tab (Ctrl + Tab)',
    },
    thumb_back: {
      id: 'thumb_back',
      label: 'Back Button',
      type: 'key_combo',
      actionName: 'Previous Tab (Ctrl + Shift + Tab)',
    },
    middle_button: {
      id: 'middle_button',
      label: 'Middle Click / Wheel Mode',
      type: 'native',
      actionName: 'Middle Click',
    },
    dpi_switch: {
      id: 'dpi_switch',
      label: 'Top Mode Shift Button',
      type: 'native',
      actionName: 'SmartShift Toggle',
    },
  } as Record<string, ButtonMapping>,
  selectedButton: null as ButtonMapping | null,

  setDpi(newDpi: number) {
    const mouse = this.devices.find((d) => d.type === 'mouse');
    if (mouse && mouse.dpi) {
      mouse.dpi.current = newDpi;
    }
    fetch('/api/set_dpi', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ dpi: newDpi }),
    }).catch((e) => console.error('Failed to set DPI:', e));
  },

  setSmartShift(enabled: boolean, threshold: number) {
    const mouse = this.devices.find((d) => d.type === 'mouse');
    if (mouse && mouse.smartshift) {
      mouse.smartshift.enabled = enabled;
      mouse.smartshift.threshold = threshold;
    }
  },

  refreshBattery() {
    const now = new Date().toLocaleTimeString();
    for (const d of this.devices) {
      d.updatedAt = now;
    }
  },
});
