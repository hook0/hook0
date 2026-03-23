const PARTICLE_COUNT = 80;
const CELEBRATION_DURATION_MS = 2500;

const CONFETTI_COLORS = [
  'var(--color-primary)',
  'var(--color-success)',
  'var(--color-warning)',
  'var(--color-error)',
  'var(--color-info)',
];

const STYLE_ID = 'celebration-styles';

function injectStyles(): void {
  if (document.getElementById(STYLE_ID)) {
    return;
  }

  const style = document.createElement('style');
  style.id = STYLE_ID;
  style.textContent = `
    .celebration-overlay {
      position: fixed;
      inset: 0;
      pointer-events: none;
      z-index: var(--z-celebration, 90);
      overflow: hidden;
    }

    .celebration-particle {
      position: absolute;
      top: -10px;
      width: 8px;
      height: 8px;
      opacity: 1;
      animation: celebration-fall linear forwards;
    }

    .celebration-particle--square {
      border-radius: 1px;
    }

    .celebration-particle--circle {
      border-radius: 50%;
    }

    .celebration-particle--strip {
      width: 4px;
      height: 12px;
      border-radius: 1px;
    }

    @keyframes celebration-fall {
      0% {
        transform: translateY(0) rotate(0deg);
        opacity: 1;
      }
      75% {
        opacity: 1;
      }
      100% {
        transform: translateY(100vh) rotate(720deg);
        opacity: 0;
      }
    }
  `;
  document.head.appendChild(style);
}

function removeStyles(): void {
  document.getElementById(STYLE_ID)?.remove();
}

function createParticle(): HTMLElement {
  const particle = document.createElement('div');
  const shapes = ['square', 'circle', 'strip'] as const;
  const shape = shapes[Math.floor(Math.random() * shapes.length)];
  const color = CONFETTI_COLORS[Math.floor(Math.random() * CONFETTI_COLORS.length)];
  const left = Math.random() * 100;
  const duration = 1.5 + Math.random() * 1.5;
  const delay = Math.random() * 0.8;

  particle.className = `celebration-particle celebration-particle--${shape}`;
  particle.style.left = `${left}%`;
  particle.style.backgroundColor = color;
  particle.style.animationDuration = `${duration}s`;
  particle.style.animationDelay = `${delay}s`;

  return particle;
}

export function useCelebration() {
  let pendingTimeout: ReturnType<typeof setTimeout> | null = null;
  let activeOverlay: HTMLElement | null = null;

  function cleanup(): void {
    if (pendingTimeout !== null) {
      clearTimeout(pendingTimeout);
      pendingTimeout = null;
    }
    if (activeOverlay) {
      activeOverlay.remove();
      activeOverlay = null;
    }
    removeStyles();
  }

  function celebrate(count: number = PARTICLE_COUNT): void {
    // Respect reduced motion preference
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
      return;
    }

    // Cancel any pending celebration before starting a new one
    cleanup();

    injectStyles();

    const overlay = document.createElement('div');
    overlay.className = 'celebration-overlay';

    for (let i = 0; i < count; i++) {
      overlay.appendChild(createParticle());
    }

    document.body.appendChild(overlay);
    activeOverlay = overlay;

    pendingTimeout = setTimeout(() => {
      cleanup();
    }, CELEBRATION_DURATION_MS + 1000);
  }

  return { celebrate, cleanup };
}
