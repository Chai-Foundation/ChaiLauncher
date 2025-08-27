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

// Function to apply color scheme by injecting custom CSS
export function applyColorScheme(settings: LauncherSettings): void {
  // Remove existing dynamic styles
  const existingStyle = document.getElementById('dynamic-colors');
  if (existingStyle) {
    existingStyle.remove();
  }

  // Generate primary and secondary palettes
  const primaryColor = settings.primary_base_color || '#78716c';
  const secondaryColor = settings.secondary_base_color || '#d97706';
  
  const primaryPalette = generateColorPalette(primaryColor);
  const secondaryPalette = generateColorPalette(secondaryColor);
  
  // Create CSS with dynamic color overrides
  let css = ':root {\n';
  
  // Add primary colors
  Object.entries(primaryPalette).forEach(([shade, color]) => {
    css += `  --primary-${shade}: ${color};\n`;
  });
  
  // Add secondary colors
  Object.entries(secondaryPalette).forEach(([shade, color]) => {
    css += `  --secondary-${shade}: ${color};\n`;
    css += `  --accent-${shade}: ${color};\n`;
  });
  
  css += '}\n\n';
  
  // Override Tailwind classes with dynamic colors
  const shades = ['50', '100', '200', '300', '400', '500', '600', '700', '800', '900', '950'];
  
  shades.forEach(shade => {
    css += `.bg-primary-${shade} { background-color: ${primaryPalette[shade]} !important; }\n`;
    css += `.text-primary-${shade} { color: ${primaryPalette[shade]} !important; }\n`;
    css += `.border-primary-${shade} { border-color: ${primaryPalette[shade]} !important; }\n`;
    css += `.ring-primary-${shade} { --tw-ring-color: ${primaryPalette[shade]} !important; }\n`;
    
    css += `.bg-secondary-${shade} { background-color: ${secondaryPalette[shade]} !important; }\n`;
    css += `.text-secondary-${shade} { color: ${secondaryPalette[shade]} !important; }\n`;
    css += `.border-secondary-${shade} { border-color: ${secondaryPalette[shade]} !important; }\n`;
    css += `.ring-secondary-${shade} { --tw-ring-color: ${secondaryPalette[shade]} !important; }\n`;
    
    // Handle opacity variants like bg-primary-800/60
    css += `.bg-primary-${shade}\\/60 { background-color: ${primaryPalette[shade]}99 !important; }\n`;
    css += `.bg-secondary-${shade}\\/60 { background-color: ${secondaryPalette[shade]}99 !important; }\n`;
    css += `.bg-primary-${shade}\\/30 { background-color: ${primaryPalette[shade]}4D !important; }\n`;
    css += `.bg-secondary-${shade}\\/30 { background-color: ${secondaryPalette[shade]}4D !important; }\n`;
    css += `.bg-primary-${shade}\\/90 { background-color: ${primaryPalette[shade]}E6 !important; }\n`;
    css += `.bg-secondary-${shade}\\/90 { background-color: ${secondaryPalette[shade]}E6 !important; }\n`;
    css += `.bg-primary-${shade}\\/50 { background-color: ${primaryPalette[shade]}80 !important; }\n`;
    css += `.bg-secondary-${shade}\\/50 { background-color: ${secondaryPalette[shade]}80 !important; }\n`;
  });
  
  // Add hover state overrides for secondary colors
  shades.forEach(shade => {
    css += `.hover\\:bg-secondary-${shade}:hover { background-color: ${secondaryPalette[shade]} !important; }\n`;
    css += `.hover\\:text-secondary-${shade}:hover { color: ${secondaryPalette[shade]} !important; }\n`;
    css += `.hover\\:border-secondary-${shade}:hover { border-color: ${secondaryPalette[shade]} !important; }\n`;
    css += `.hover\\:from-secondary-${shade}:hover { --tw-gradient-from: ${secondaryPalette[shade]} !important; }\n`;
    css += `.hover\\:to-secondary-${shade}:hover { --tw-gradient-to: ${secondaryPalette[shade]} !important; }\n`;
  });
  
  // Add gradient color overrides
  shades.forEach(shade => {
    css += `.from-secondary-${shade} { --tw-gradient-from: ${secondaryPalette[shade]} !important; --tw-gradient-to: transparent !important; --tw-gradient-stops: var(--tw-gradient-from), var(--tw-gradient-to) !important; }\n`;
    css += `.to-secondary-${shade} { --tw-gradient-to: ${secondaryPalette[shade]} !important; }\n`;
    css += `.via-secondary-${shade} { --tw-gradient-to: transparent !important; --tw-gradient-stops: var(--tw-gradient-from), ${secondaryPalette[shade]}, var(--tw-gradient-to) !important; }\n`;
    
    // Add more opacity variants
    css += `.bg-secondary-${shade}\/20 { background-color: ${secondaryPalette[shade]}33 !important; }\n`;
    css += `.bg-secondary-${shade}\/80 { background-color: ${secondaryPalette[shade]}CC !important; }\n`;
    css += `.border-secondary-${shade}\/30 { border-color: ${secondaryPalette[shade]}4D !important; }\n`;
    css += `.border-secondary-${shade}\/50 { border-color: ${secondaryPalette[shade]}80 !important; }\n`;
  });
  
  // Inject the CSS into the document
  const styleElement = document.createElement('style');
  styleElement.id = 'dynamic-colors';
  styleElement.textContent = css;
  document.head.appendChild(styleElement);
}

// Function to get the current color for display purposes
export function getCurrentPrimaryColor(settings: LauncherSettings): string {
  return settings.primary_base_color || '#78716c';
}

export function getCurrentSecondaryColor(settings: LauncherSettings): string {
  return settings.secondary_base_color || '#d97706';
}