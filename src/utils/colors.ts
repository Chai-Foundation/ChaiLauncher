import { LauncherSettings } from '../types/minecraft';

// Function to convert hex to HSL
function hexToHsl(hex: string): [number, number, number] {
  const r = parseInt(hex.slice(1, 3), 16) / 255;
  const g = parseInt(hex.slice(3, 5), 16) / 255;
  const b = parseInt(hex.slice(5, 7), 16) / 255;

  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  let h: number, s: number, l: number;

  l = (max + min) / 2;

  if (max === min) {
    h = s = 0;
  } else {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    switch (max) {
      case r: h = (g - b) / d + (g < b ? 6 : 0); break;
      case g: h = (b - r) / d + 2; break;
      case b: h = (r - g) / d + 4; break;
      default: h = 0;
    }
    h /= 6;
  }

  return [h * 360, s * 100, l * 100];
}

// Function to convert HSL to hex
function hslToHex(h: number, s: number, l: number): string {
  h = h / 360;
  s = s / 100;
  l = l / 100;

  const hue2rgb = (p: number, q: number, t: number) => {
    if (t < 0) t += 1;
    if (t > 1) t -= 1;
    if (t < 1/6) return p + (q - p) * 6 * t;
    if (t < 1/2) return q;
    if (t < 2/3) return p + (q - p) * (2/3 - t) * 6;
    return p;
  };

  let r: number, g: number, b: number;

  if (s === 0) {
    r = g = b = l;
  } else {
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    const p = 2 * l - q;
    r = hue2rgb(p, q, h + 1/3);
    g = hue2rgb(p, q, h);
    b = hue2rgb(p, q, h - 1/3);
  }

  const toHex = (c: number) => {
    const hex = Math.round(c * 255).toString(16);
    return hex.length === 1 ? '0' + hex : hex;
  };

  return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
}

// Function to generate a complete color palette from a base color
function generateColorPalette(baseColor: string): Record<string, string> {
  const [h, s, l] = hexToHsl(baseColor);
  
  return {
    '50': hslToHex(h, Math.max(s - 40, 5), Math.min(l + 45, 97)),
    '100': hslToHex(h, Math.max(s - 30, 10), Math.min(l + 40, 95)),
    '200': hslToHex(h, Math.max(s - 20, 15), Math.min(l + 30, 90)),
    '300': hslToHex(h, Math.max(s - 10, 20), Math.min(l + 20, 85)),
    '400': hslToHex(h, s, Math.min(l + 10, 75)),
    '500': baseColor, // The base color
    '600': hslToHex(h, Math.min(s + 10, 90), Math.max(l - 10, 15)),
    '700': hslToHex(h, Math.min(s + 15, 95), Math.max(l - 20, 10)),
    '800': hslToHex(h, Math.min(s + 20, 100), Math.max(l - 30, 8)),
    '900': hslToHex(h, Math.min(s + 25, 100), Math.max(l - 40, 5)),
    '950': hslToHex(h, Math.min(s + 30, 100), Math.max(l - 50, 3)),
  };
}

// Function to apply color scheme to CSS custom properties
export function applyColorScheme(settings: LauncherSettings): void {
  const root = document.documentElement;
  
  // Determine which color scheme to use as primary
  const primaryColor = settings.color_scheme === 'amber' 
    ? settings.amber_base_color || '#d97706'
    : settings.stone_base_color || '#78716c';
    
  // Generate primary palette
  const primaryPalette = generateColorPalette(primaryColor);
  
  // Apply primary colors
  Object.entries(primaryPalette).forEach(([shade, color]) => {
    root.style.setProperty(`--primary-${shade}`, color);
  });
  
  // Always use amber as accent color (but allow customization)
  const accentColor = settings.amber_base_color || '#d97706';
  const accentPalette = generateColorPalette(accentColor);
  
  // Apply accent colors
  Object.entries(accentPalette).forEach(([shade, color]) => {
    root.style.setProperty(`--accent-${shade}`, color);
  });
  
  // Apply background image if specified
  if (settings.background_image) {
    root.style.setProperty('--bg-image', `url("${settings.background_image}")`);
  } else {
    root.style.removeProperty('--bg-image');
  }
}

// Function to get the current color for display purposes
export function getCurrentPrimaryColor(settings: LauncherSettings): string {
  return settings.color_scheme === 'amber' 
    ? settings.amber_base_color || '#d97706'
    : settings.stone_base_color || '#78716c';
}

export function getCurrentAccentColor(settings: LauncherSettings): string {
  return settings.amber_base_color || '#d97706';
}