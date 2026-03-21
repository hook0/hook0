<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

const showFish = ref(false);
let fishTimeout: number;

onMounted(() => {
  fishTimeout = window.setTimeout(() => {
    showFish.value = true;
  }, 2000);
});

onUnmounted(() => {
  clearTimeout(fishTimeout);
});
</script>

<template>
  <div class="hook0-error404-bg">
    <!-- Underwater background effects -->
    <div class="hook0-error404-bg__underwater">
      <div class="hook0-error404-bg__bubble hook0-error404-bg__bubble--1"></div>
      <div class="hook0-error404-bg__bubble hook0-error404-bg__bubble--2"></div>
      <div class="hook0-error404-bg__bubble hook0-error404-bg__bubble--3"></div>
      <div class="hook0-error404-bg__bubble hook0-error404-bg__bubble--4"></div>
      <div class="hook0-error404-bg__bubble hook0-error404-bg__bubble--5"></div>
      <div class="hook0-error404-bg__bubble hook0-error404-bg__bubble--6"></div>
      <div class="hook0-error404-bg__bubble hook0-error404-bg__bubble--7"></div>
      <div class="hook0-error404-bg__ray hook0-error404-bg__ray--1"></div>
      <div class="hook0-error404-bg__ray hook0-error404-bg__ray--2"></div>
      <div class="hook0-error404-bg__ray hook0-error404-bg__ray--3"></div>
    </div>

    <!-- Fish swimming away -->
    <div v-if="showFish" class="hook0-error404-bg__fish-container">
      <div class="hook0-error404-bg__fish hook0-error404-bg__fish--1">
        <svg viewBox="0 0 40 24" class="hook0-error404-bg__fish-svg">
          <path
            d="M35 12 Q40 12 35 6 L20 12 L35 18 Q40 12 35 12 M20 12 L5 6 Q0 12 5 18 L20 12"
            fill="currentColor"
          />
          <circle cx="8" cy="11" r="2" fill="#0a0a0f" />
        </svg>
      </div>
      <div class="hook0-error404-bg__fish hook0-error404-bg__fish--2">
        <svg viewBox="0 0 40 24" class="hook0-error404-bg__fish-svg">
          <path
            d="M35 12 Q40 12 35 6 L20 12 L35 18 Q40 12 35 12 M20 12 L5 6 Q0 12 5 18 L20 12"
            fill="currentColor"
          />
          <circle cx="8" cy="11" r="2" fill="#0a0a0f" />
        </svg>
      </div>
      <div class="hook0-error404-bg__fish hook0-error404-bg__fish--3">
        <svg viewBox="0 0 40 24" class="hook0-error404-bg__fish-svg">
          <path
            d="M35 12 Q40 12 35 6 L20 12 L35 18 Q40 12 35 12 M20 12 L5 6 Q0 12 5 18 L20 12"
            fill="currentColor"
          />
          <circle cx="8" cy="11" r="2" fill="#0a0a0f" />
        </svg>
      </div>
    </div>
  </div>
</template>

<style scoped>
.hook0-error404-bg {
  position: absolute;
  inset: 0;
  background: linear-gradient(180deg, #0a1628 0%, #0a0a0f 50%, #0d1a2d 100%);
}

/* Underwater background */
.hook0-error404-bg__underwater {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

/* Floating bubbles */
.hook0-error404-bg__bubble {
  position: absolute;
  border-radius: 9999px;
  background: radial-gradient(
    circle at 30% 30%,
    rgba(255, 255, 255, 0.4),
    rgba(255, 255, 255, 0.1)
  );
  animation: bubble-rise linear infinite;
}

.hook0-error404-bg__bubble--1 {
  width: 20px;
  height: 20px;
  left: 10%;
  animation-duration: 8s;
  animation-delay: 0s;
}

.hook0-error404-bg__bubble--2 {
  width: 15px;
  height: 15px;
  left: 25%;
  animation-duration: 10s;
  animation-delay: 2s;
}

.hook0-error404-bg__bubble--3 {
  width: 25px;
  height: 25px;
  left: 45%;
  animation-duration: 12s;
  animation-delay: 1s;
}

.hook0-error404-bg__bubble--4 {
  width: 12px;
  height: 12px;
  left: 65%;
  animation-duration: 9s;
  animation-delay: 3s;
}

.hook0-error404-bg__bubble--5 {
  width: 18px;
  height: 18px;
  left: 80%;
  animation-duration: 11s;
  animation-delay: 0.5s;
}

.hook0-error404-bg__bubble--6 {
  width: 10px;
  height: 10px;
  left: 35%;
  animation-duration: 7s;
  animation-delay: 4s;
}

.hook0-error404-bg__bubble--7 {
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

/* Light rays from surface */
.hook0-error404-bg__ray {
  position: absolute;
  top: 0;
  background: linear-gradient(
    180deg,
    color-mix(in srgb, var(--color-primary) 15%, transparent) 0%,
    transparent 100%
  );
  transform-origin: top center;
  animation: ray-sway 8s ease-in-out infinite;
}

.hook0-error404-bg__ray--1 {
  left: 20%;
  width: 100px;
  height: 60%;
  transform: rotate(-15deg);
  animation-delay: 0s;
  --initial-rotation: -15deg;
}

.hook0-error404-bg__ray--2 {
  left: 50%;
  width: 150px;
  height: 70%;
  transform: rotate(5deg);
  animation-delay: 2s;
  --initial-rotation: 5deg;
}

.hook0-error404-bg__ray--3 {
  left: 75%;
  width: 80px;
  height: 50%;
  transform: rotate(20deg);
  animation-delay: 4s;
  --initial-rotation: 20deg;
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

/* Fish swimming away */
.hook0-error404-bg__fish-container {
  position: absolute;
  inset: 0;
  pointer-events: none;
  overflow: hidden;
}

.hook0-error404-bg__fish {
  position: absolute;
  animation: swim-away 4s ease-out forwards;
}

.hook0-error404-bg__fish--1 {
  top: 35%;
  left: 60%;
  animation-delay: 0s;
}

.hook0-error404-bg__fish--2 {
  top: 45%;
  left: 55%;
  animation-delay: 0.3s;
}

.hook0-error404-bg__fish--3 {
  top: 40%;
  left: 65%;
  animation-delay: 0.6s;
}

.hook0-error404-bg__fish-svg {
  width: 40px;
  height: 24px;
  color: var(--color-primary);
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
</style>
