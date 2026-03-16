const PARTICLE_COUNT = 80;
const CELEBRATION_DURATION_MS = 2500;

const CONFETTI_COLORS = [
  'var(--color-primary)',
  'var(--color-success)',
  'var(--color-warning)',
  'var(--color-error)',
  'var(--color-info)',
];

function injectStyles(): void {
  if (document.getElementById('celebration-styles')) {
    return;
  }

  const style = document.createElement('style');
  style.id = 'celebration-styles';
  style.textContent = `
    .celebration-overlay {
      position: fixed;
      inset: 0;
      pointer-events: none;
      z-index: 9999;
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
  function celebrate(count: number = PARTICLE_COUNT): void {
    injectStyles();

    const overlay = document.createElement('div');
    overlay.className = 'celebration-overlay';

    for (let i = 0; i < count; i++) {
      overlay.appendChild(createParticle());
    }

    document.body.appendChild(overlay);

    setTimeout(() => {
      overlay.remove();
    }, CELEBRATION_DURATION_MS + 1000);
  }

  return { celebrate };
}
