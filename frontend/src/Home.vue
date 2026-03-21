<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import OrganizationSelector from '@/pages/OrganizationAndApplicationSelector.vue';

const { t } = useI18n();
</script>

<template>
  <div class="home" data-test="home-page">
    <!-- Welcome Banner -->
    <div class="home__banner" data-test="home-banner">
      <div class="home__banner-content">
        <h1 class="home__title">{{ t('home.welcome') }}</h1>
        <p class="home__subtitle">
          <i18n-t keypath="home.gettingStartedSubtitle" tag="span">
            <template #link>
              <a
                href="https://documentation.hook0.com/docs/getting-started"
                target="_blank"
                rel="noopener noreferrer"
                class="home__link"
                >{{ t('home.gettingStarted') }}</a
              >
            </template>
          </i18n-t>
        </p>
      </div>
    </div>

    <!-- Section Header -->
    <div class="home__section-header">
      <h2 class="home__section-title">{{ t('home.yourOrganizations') }}</h2>
      <p class="home__section-subtitle">{{ t('home.organizationsSubtitle') }}</p>
    </div>

    <!-- Organization Cards -->
    <OrganizationSelector :display-as-cards="true" />
  </div>
</template>

<style scoped>
.home {
  max-width: 80rem;
  margin: 0 auto;
  padding: 2rem 1.5rem 4rem;
}

.home__banner {
  position: relative;
  padding: 2rem 2.5rem;
  border-radius: var(--radius-xl);
  background-color: var(--color-primary);
  margin-bottom: 2.5rem;
  overflow: hidden;
}

.home__banner::before {
  content: '';
  position: absolute;
  inset: 0;
  background-image:
    radial-gradient(
      circle at 20% 80%,
      color-mix(in srgb, var(--color-on-dark) 8%, transparent) 0%,
      transparent 50%
    ),
    radial-gradient(
      circle at 80% 20%,
      color-mix(in srgb, var(--color-on-dark) 6%, transparent) 0%,
      transparent 50%
    );
  pointer-events: none;
}

.home__banner-content {
  position: relative;
  z-index: 1;
}

.home__title {
  font-size: 1.875rem;
  font-weight: 800;
  color: var(--color-on-dark);
  margin-bottom: 0.5rem;
  letter-spacing: -0.02em;
}

.home__subtitle {
  font-size: 1.0625rem;
  font-weight: 500;
  color: color-mix(in srgb, var(--color-on-dark) 90%, transparent);
  line-height: 1.6;
}

.home__link {
  color: var(--color-on-dark);
  font-weight: 600;
  text-decoration: underline;
  text-underline-offset: 2px;
  transition: opacity 0.15s ease;
}

.home__link:hover {
  opacity: 0.85;
}

.home__section-header {
  margin-bottom: 1.5rem;
}

.home__section-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: 0.25rem;
}

.home__section-subtitle {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

@media (max-width: 640px) {
  .home {
    padding: 1rem 1rem 3rem;
  }

  .home__banner {
    padding: 1.5rem;
  }

  .home__title {
    font-size: 1.5rem;
  }
}
</style>
