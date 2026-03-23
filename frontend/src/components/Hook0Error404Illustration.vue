<script setup lang="ts">
// Animated fishing hook illustration for 404 error pages
// Provides a playful "nothing caught" metaphor for webhook-themed apps
</script>

<template>
  <div class="error-illustration">
    <!-- The fishing hook animation -->
    <div class="hook-container">
      <div class="fishing-line"></div>
      <div class="hook">
        <svg
          viewBox="0 0 60 100"
          class="hook-svg"
          fill="none"
          stroke="currentColor"
          stroke-width="4"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="M30 0 L30 50 Q30 70 45 75 Q60 80 55 60 Q52 45 40 50" class="hook-path" />
          <path d="M40 50 L35 45" class="hook-barb" />
        </svg>
        <div class="empty-catch">
          <span>?</span>
        </div>
      </div>
    </div>

    <!-- 404 Text -->
    <h1 class="error-code">
      <span class="digit digit-4-1">4</span>
      <span class="digit digit-0">0</span>
      <span class="digit digit-4-2">4</span>
    </h1>
  </div>
</template>

<style scoped>
.error-illustration {
  display: flex;
  flex-direction: column;
  align-items: center;
}

/* Hook animation container */
.hook-container {
  position: relative;
  margin-bottom: 1rem;
  width: 100px;
  height: 150px;
  animation: hook-swing 3s ease-in-out infinite;
  transform-origin: top center;
}

.fishing-line {
  position: absolute;
  left: 50%;
  top: 0;
  width: 3px;
  height: 50px;
  background: linear-gradient(180deg, rgba(156, 163, 175, 0.8), rgba(156, 163, 175, 0.4));
  transform: translateX(-50%);
}

.hook {
  position: absolute;
  left: 50%;
  top: 45px;
  transform: translateX(-50%);
  animation: hook-bob 2s ease-in-out infinite;
}

.hook-svg {
  width: 50px;
  height: 80px;
  color: var(--color-text-muted);
  filter: drop-shadow(0 0 10px color-mix(in srgb, var(--color-primary) 30%, transparent));
}

.hook-path {
  stroke-dasharray: 200;
  stroke-dashoffset: 200;
  animation: hook-draw 2s ease-out forwards;
}

.hook-barb {
  stroke-dasharray: 20;
  stroke-dashoffset: 20;
  animation: hook-draw 1s ease-out 1.5s forwards;
}

@keyframes hook-draw {
  to {
    stroke-dashoffset: 0;
  }
}

@keyframes hook-swing {
  0%,
  100% {
    transform: rotate(-8deg);
  }
  50% {
    transform: rotate(8deg);
  }
}

@keyframes hook-bob {
  0%,
  100% {
    transform: translateX(-50%) translateY(0);
  }
  50% {
    transform: translateX(-50%) translateY(8px);
  }
}

/* Empty catch indicator */
.empty-catch {
  position: absolute;
  right: -0.75rem;
  top: 2.5rem;
  width: 1.75rem;
  height: 1.75rem;
  border-radius: 9999px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1rem;
  font-weight: 700;
  background: color-mix(in srgb, var(--color-warning) 20%, transparent);
  color: var(--color-warning);
  animation: question-pulse 2s ease-in-out infinite;
}

@keyframes question-pulse {
  0%,
  100% {
    transform: scale(1);
    opacity: 1;
  }
  50% {
    transform: scale(1.2);
    opacity: 0.7;
  }
}

/* Error code */
.error-code {
  font-size: 4.5rem;
  font-weight: 900;
  margin: 0;
  letter-spacing: -0.025em;
  background: linear-gradient(
    135deg,
    var(--color-primary) 0%,
    var(--color-primary-gradient-end) 50%,
    var(--color-primary) 100%
  );
  background-size: 200% 200%;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  animation: gradient-shift 4s ease infinite;
  text-shadow: 0 0 60px color-mix(in srgb, var(--color-primary) 50%, transparent);
}

.digit {
  display: inline-block;
  animation: digit-float 3s ease-in-out infinite;
}

.digit-4-1 {
  animation-delay: 0s;
}

.digit-0 {
  animation-delay: 0.2s;
}

.digit-4-2 {
  animation-delay: 0.4s;
}

@keyframes digit-float {
  0%,
  100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-10px);
  }
}

@keyframes gradient-shift {
  0%,
  100% {
    background-position: 0% 50%;
  }
  50% {
    background-position: 100% 50%;
  }
}
</style>
