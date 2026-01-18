<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();
const retryCount = ref(0);
const maxRetries = 5;
const isGivingUp = ref(false);
const showFish = ref(false);

let retryInterval: number;
let fishTimeout: number;

onMounted(() => {
  // Simulate webhook retry attempts
  retryInterval = window.setInterval(() => {
    if (retryCount.value < maxRetries) {
      retryCount.value++;
    } else {
      isGivingUp.value = true;
      clearInterval(retryInterval);
    }
  }, 1500);

  // Show fish swimming away after a delay
  fishTimeout = window.setTimeout(() => {
    showFish.value = true;
  }, 2000);
});

onUnmounted(() => {
  clearInterval(retryInterval);
  clearTimeout(fishTimeout);
});

function goHome() {
  router.push('/').catch(() => {
    // Navigation cancelled or failed - ignore
  });
}

function goBack() {
  router.back();
}
</script>

<template>
  <div class="error-page">
    <!-- Underwater background effects -->
    <div class="underwater-bg">
      <div class="bubble bubble-1"></div>
      <div class="bubble bubble-2"></div>
      <div class="bubble bubble-3"></div>
      <div class="bubble bubble-4"></div>
      <div class="bubble bubble-5"></div>
      <div class="bubble bubble-6"></div>
      <div class="bubble bubble-7"></div>
      <div class="light-ray light-ray-1"></div>
      <div class="light-ray light-ray-2"></div>
      <div class="light-ray light-ray-3"></div>
    </div>

    <!-- Main content -->
    <div class="content">
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
          >
            <!-- The hook shape -->
            <path d="M30 0 L30 50 Q30 70 45 75 Q60 80 55 60 Q52 45 40 50" class="hook-path" />
            <!-- Little barb -->
            <path d="M40 50 L35 45" class="hook-barb" />
          </svg>
          <!-- Empty catch indicator -->
          <div class="empty-catch">
            <span>?</span>
          </div>
        </div>
      </div>

      <!-- Fish swimming away -->
      <div v-if="showFish" class="fish-container">
        <div class="fish fish-1">
          <svg viewBox="0 0 40 24" class="fish-svg">
            <path
              d="M35 12 Q40 12 35 6 L20 12 L35 18 Q40 12 35 12 M20 12 L5 6 Q0 12 5 18 L20 12"
              fill="currentColor"
            />
            <circle cx="8" cy="11" r="2" fill="#0a0a0f" />
          </svg>
        </div>
        <div class="fish fish-2">
          <svg viewBox="0 0 40 24" class="fish-svg">
            <path
              d="M35 12 Q40 12 35 6 L20 12 L35 18 Q40 12 35 12 M20 12 L5 6 Q0 12 5 18 L20 12"
              fill="currentColor"
            />
            <circle cx="8" cy="11" r="2" fill="#0a0a0f" />
          </svg>
        </div>
        <div class="fish fish-3">
          <svg viewBox="0 0 40 24" class="fish-svg">
            <path
              d="M35 12 Q40 12 35 6 L20 12 L35 18 Q40 12 35 12 M20 12 L5 6 Q0 12 5 18 L20 12"
              fill="currentColor"
            />
            <circle cx="8" cy="11" r="2" fill="#0a0a0f" />
          </svg>
        </div>
      </div>

      <!-- 404 Text -->
      <div class="error-text">
        <h1 class="error-code">
          <span class="digit digit-4-1">4</span>
          <span class="digit digit-0">0</span>
          <span class="digit digit-4-2">4</span>
        </h1>
        <h2 class="error-title">This Hook Came Up Empty</h2>
        <p class="error-description">The endpoint you're looking for swam away...</p>
      </div>

      <!-- Retry counter (webhook humor) -->
      <div class="retry-status">
        <div v-if="!isGivingUp" class="retry-active">
          <div class="retry-spinner"></div>
          <span>Retry attempt {{ retryCount }}/{{ maxRetries }}...</span>
        </div>
        <div v-else class="retry-failed">
          <span class="retry-icon">&#x2715;</span>
          <span>Max retries exceeded. The page doesn't exist!</span>
        </div>
      </div>

      <!-- Action buttons -->
      <div class="actions">
        <button class="btn btn-primary" @click="goHome">
          <svg
            class="btn-icon"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path
              d="M3 12l9-9 9 9M5 10v10a1 1 0 001 1h3m10-11v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1"
            />
          </svg>
          Go to Dashboard
        </button>
        <button class="btn btn-secondary" @click="goBack">
          <svg
            class="btn-icon"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path d="M19 12H5m0 0l7 7m-7-7l7-7" />
          </svg>
          Go Back
        </button>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.error-page {
  @apply min-h-screen flex items-center justify-center relative overflow-hidden;
  background: linear-gradient(180deg, #0a1628 0%, #0a0a0f 50%, #0d1a2d 100%);
}

// Underwater background
.underwater-bg {
  @apply absolute inset-0 pointer-events-none;
}

// Floating bubbles
.bubble {
  @apply absolute rounded-full;
  background: radial-gradient(
    circle at 30% 30%,
    rgba(255, 255, 255, 0.4),
    rgba(255, 255, 255, 0.1)
  );
  animation: bubble-rise linear infinite;
}

.bubble-1 {
  width: 20px;
  height: 20px;
  left: 10%;
  animation-duration: 8s;
  animation-delay: 0s;
}
.bubble-2 {
  width: 15px;
  height: 15px;
  left: 25%;
  animation-duration: 10s;
  animation-delay: 2s;
}
.bubble-3 {
  width: 25px;
  height: 25px;
  left: 45%;
  animation-duration: 12s;
  animation-delay: 1s;
}
.bubble-4 {
  width: 12px;
  height: 12px;
  left: 65%;
  animation-duration: 9s;
  animation-delay: 3s;
}
.bubble-5 {
  width: 18px;
  height: 18px;
  left: 80%;
  animation-duration: 11s;
  animation-delay: 0.5s;
}
.bubble-6 {
  width: 10px;
  height: 10px;
  left: 35%;
  animation-duration: 7s;
  animation-delay: 4s;
}
.bubble-7 {
  width: 22px;
  height: 22px;
  left: 55%;
  animation-duration: 13s;
  animation-delay: 2.5s;
}

@keyframes bubble-rise {
  0% {
    bottom: -50px;
    opacity: 0;
    transform: translateX(0) scale(0.5);
  }
  10% {
    opacity: 0.6;
    transform: translateX(10px) scale(1);
  }
  50% {
    transform: translateX(-15px) scale(1);
  }
  90% {
    opacity: 0.4;
    transform: translateX(10px) scale(0.9);
  }
  100% {
    bottom: 110%;
    opacity: 0;
    transform: translateX(0) scale(0.7);
  }
}

// Light rays from surface
.light-ray {
  @apply absolute top-0;
  background: linear-gradient(180deg, rgba(99, 102, 241, 0.15) 0%, transparent 100%);
  transform-origin: top center;
  animation: ray-sway 8s ease-in-out infinite;
}

.light-ray-1 {
  left: 20%;
  width: 100px;
  height: 60%;
  transform: rotate(-15deg);
  animation-delay: 0s;
}
.light-ray-2 {
  left: 50%;
  width: 150px;
  height: 70%;
  transform: rotate(5deg);
  animation-delay: 2s;
}
.light-ray-3 {
  left: 75%;
  width: 80px;
  height: 50%;
  transform: rotate(20deg);
  animation-delay: 4s;
}

@keyframes ray-sway {
  0%,
  100% {
    transform: rotate(var(--initial-rotation, 0deg));
    opacity: 0.5;
  }
  50% {
    transform: rotate(calc(var(--initial-rotation, 0deg) + 5deg));
    opacity: 0.8;
  }
}

.light-ray-1 {
  --initial-rotation: -15deg;
}
.light-ray-2 {
  --initial-rotation: 5deg;
}
.light-ray-3 {
  --initial-rotation: 20deg;
}

// Main content
.content {
  @apply relative z-10 text-center px-4;
}

// Hook animation container
.hook-container {
  @apply relative mx-auto mb-8;
  width: 100px;
  height: 180px;
  animation: hook-swing 3s ease-in-out infinite;
  transform-origin: top center;
}

.fishing-line {
  @apply absolute left-1/2 top-0;
  width: 3px;
  height: 60px;
  background: linear-gradient(180deg, rgba(156, 163, 175, 0.8), rgba(156, 163, 175, 0.4));
  transform: translateX(-50%);
}

.hook {
  @apply absolute left-1/2;
  top: 55px;
  transform: translateX(-50%);
  animation: hook-bob 2s ease-in-out infinite;
}

.hook-svg {
  width: 60px;
  height: 100px;
  color: #9ca3af;
  filter: drop-shadow(0 0 10px rgba(99, 102, 241, 0.3));
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

// Empty catch indicator
.empty-catch {
  @apply absolute -right-4 top-12 w-8 h-8 rounded-full flex items-center justify-center text-lg font-bold;
  background: rgba(251, 191, 36, 0.2);
  color: #fbbf24;
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

// Fish swimming away
.fish-container {
  @apply absolute inset-0 pointer-events-none overflow-hidden;
}

.fish {
  @apply absolute;
  animation: swim-away 4s ease-out forwards;
}

.fish-1 {
  top: 35%;
  left: 60%;
  animation-delay: 0s;
}
.fish-2 {
  top: 45%;
  left: 55%;
  animation-delay: 0.3s;
}
.fish-3 {
  top: 40%;
  left: 65%;
  animation-delay: 0.6s;
}

.fish-svg {
  width: 40px;
  height: 24px;
  color: #22c55e;
  opacity: 0.8;
}

@keyframes swim-away {
  0% {
    transform: translateX(0) scaleX(-1);
    opacity: 0;
  }
  10% {
    opacity: 0.8;
  }
  100% {
    transform: translateX(calc(50vw)) scaleX(-1);
    opacity: 0;
  }
}

// Error text
.error-text {
  @apply mb-8;
}

.error-code {
  @apply text-8xl font-black mb-4 tracking-tight;
  background: linear-gradient(135deg, #6366f1 0%, #22c55e 50%, #6366f1 100%);
  background-size: 200% 200%;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  animation: gradient-shift 4s ease infinite;
  text-shadow: 0 0 60px rgba(99, 102, 241, 0.5);
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

.error-title {
  @apply text-2xl font-semibold text-white mb-3;
}

.error-description {
  @apply text-gray-400 text-lg;
}

// Retry status (webhook humor)
.retry-status {
  @apply mb-8 text-sm;
}

.retry-active {
  @apply flex items-center justify-center gap-2 text-indigo-400;
}

.retry-spinner {
  @apply w-4 h-4 rounded-full border-2 border-indigo-400 border-t-transparent;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.retry-failed {
  @apply flex items-center justify-center gap-2 text-yellow-400;
  animation: fade-in 0.5s ease-out;
}

.retry-icon {
  @apply text-lg;
}

@keyframes fade-in {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

// Action buttons
.actions {
  @apply flex flex-col sm:flex-row gap-4 justify-center;
}

.btn {
  @apply inline-flex items-center justify-center gap-2 px-6 py-3 rounded-lg font-medium transition-all duration-300;

  &:hover {
    transform: translateY(-2px);
  }

  &:active {
    transform: translateY(0);
  }
}

.btn-primary {
  @apply bg-green-500 text-white;
  box-shadow:
    0 4px 14px rgba(34, 197, 94, 0.4),
    0 0 40px rgba(34, 197, 94, 0.2);

  &:hover {
    @apply bg-green-400;
    box-shadow:
      0 6px 20px rgba(34, 197, 94, 0.5),
      0 0 60px rgba(34, 197, 94, 0.3);
  }
}

.btn-secondary {
  @apply bg-white/10 text-white border border-white/20;
  backdrop-filter: blur(10px);

  &:hover {
    @apply bg-white/20 border-white/30;
  }
}

.btn-icon {
  @apply w-5 h-5;
}
</style>
