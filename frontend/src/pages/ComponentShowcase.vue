<script setup lang="ts">
/**
 * Component showcase page for visual regression testing.
 * Only available in development mode via /__dev/components route.
 *
 * Renders all Hook0Button variants, sizes, and states for Playwright screenshot testing.
 */
import { ArrowRight, Trash2, Plus, Settings, ExternalLink } from 'lucide-vue-next';
import Hook0Button from '@/components/Hook0Button.vue';
</script>

<template>
  <div class="showcase" data-test="component-showcase">
    <h1 class="showcase__title">Component Showcase</h1>

    <!-- Button Variants -->
    <section class="showcase__section" data-test="button-variants">
      <h2 class="showcase__section-title">Button Variants</h2>
      <div class="showcase__row">
        <Hook0Button variant="primary" data-test="btn-primary"> Primary </Hook0Button>
        <Hook0Button variant="secondary" data-test="btn-secondary"> Secondary </Hook0Button>
        <Hook0Button variant="danger" data-test="btn-danger"> Danger </Hook0Button>
        <Hook0Button variant="ghost" data-test="btn-ghost"> Ghost </Hook0Button>
        <Hook0Button variant="link" data-test="btn-link"> Link </Hook0Button>
        <Hook0Button variant="icon" data-test="btn-icon">
          <Settings :size="16" aria-hidden="true" />
        </Hook0Button>
      </div>
    </section>

    <!-- Button Sizes -->
    <section class="showcase__section" data-test="button-sizes">
      <h2 class="showcase__section-title">Button Sizes</h2>
      <div class="showcase__row">
        <Hook0Button variant="primary" size="sm" data-test="btn-size-sm"> Small </Hook0Button>
        <Hook0Button variant="primary" size="md" data-test="btn-size-md"> Medium </Hook0Button>
        <Hook0Button variant="primary" size="lg" data-test="btn-size-lg"> Large </Hook0Button>
      </div>
    </section>

    <!-- Buttons with Icons -->
    <section class="showcase__section" data-test="button-icons">
      <h2 class="showcase__section-title">Buttons with Icons</h2>
      <div class="showcase__row">
        <Hook0Button variant="primary" data-test="btn-icon-right">
          <template #right>
            <ArrowRight :size="16" aria-hidden="true" />
          </template>
          Continue
        </Hook0Button>
        <Hook0Button variant="danger" data-test="btn-icon-left">
          <template #left>
            <Trash2 :size="16" aria-hidden="true" />
          </template>
          Delete
        </Hook0Button>
        <Hook0Button variant="secondary" data-test="btn-icon-both">
          <template #left>
            <Plus :size="16" aria-hidden="true" />
          </template>
          <template #right>
            <ArrowRight :size="16" aria-hidden="true" />
          </template>
          Create
        </Hook0Button>
      </div>
    </section>

    <!-- Disabled States -->
    <section class="showcase__section" data-test="button-disabled">
      <h2 class="showcase__section-title">Disabled States</h2>
      <div class="showcase__row">
        <Hook0Button variant="primary" disabled data-test="btn-disabled-primary">
          Disabled Primary
        </Hook0Button>
        <Hook0Button variant="secondary" disabled data-test="btn-disabled-secondary">
          Disabled Secondary
        </Hook0Button>
        <Hook0Button variant="danger" disabled data-test="btn-disabled-danger">
          Disabled Danger
        </Hook0Button>
      </div>
    </section>

    <!-- Loading States -->
    <section class="showcase__section" data-test="button-loading">
      <h2 class="showcase__section-title">Loading States</h2>
      <div class="showcase__row">
        <Hook0Button variant="primary" :loading="true" data-test="btn-loading-primary">
          Loading Primary
        </Hook0Button>
        <Hook0Button variant="secondary" :loading="true" data-test="btn-loading-secondary">
          Loading Secondary
        </Hook0Button>
        <Hook0Button variant="danger" :loading="true" data-test="btn-loading-danger">
          Loading Danger
        </Hook0Button>
      </div>
    </section>

    <!-- Full Width -->
    <section class="showcase__section" data-test="button-fullwidth">
      <h2 class="showcase__section-title">Full Width</h2>
      <div class="showcase__column">
        <Hook0Button variant="primary" full-width data-test="btn-fullwidth-primary">
          Full Width Primary
        </Hook0Button>
        <Hook0Button variant="secondary" full-width data-test="btn-fullwidth-secondary">
          Full Width Secondary
        </Hook0Button>
      </div>
    </section>

    <!-- Submit Button (renders as <button type="submit">) -->
    <section class="showcase__section" data-test="button-submit">
      <h2 class="showcase__section-title">Submit Buttons</h2>
      <div class="showcase__row">
        <Hook0Button variant="primary" submit data-test="btn-submit-primary">
          Submit Primary
        </Hook0Button>
        <Hook0Button variant="danger" submit data-test="btn-submit-danger">
          Submit Danger
        </Hook0Button>
      </div>
    </section>

    <!-- Link Buttons (renders as <a>) -->
    <section class="showcase__section" data-test="button-links">
      <h2 class="showcase__section-title">Link Buttons (renders as &lt;a&gt;)</h2>
      <div class="showcase__row">
        <Hook0Button variant="primary" href="https://example.com" data-test="btn-link-href">
          <template #right>
            <ExternalLink :size="16" aria-hidden="true" />
          </template>
          External Link
        </Hook0Button>
        <Hook0Button variant="secondary" :to="{ path: '/login' }" data-test="btn-link-to">
          Router Link
        </Hook0Button>
      </div>
    </section>

    <!-- Element Type Verification -->
    <section class="showcase__section" data-test="button-elements">
      <h2 class="showcase__section-title">Element Type Verification</h2>
      <p class="showcase__hint">
        Action buttons render as &lt;button&gt;, navigation buttons render as &lt;a&gt;
      </p>
      <div class="showcase__row">
        <Hook0Button variant="primary" data-test="btn-element-button"> I am a button </Hook0Button>
        <Hook0Button variant="primary" href="https://example.com" data-test="btn-element-anchor">
          I am an anchor
        </Hook0Button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.showcase {
  max-width: 800px;
  margin: 2rem auto;
  padding: 2rem;
  font-family: var(--font-sans);
}

.showcase__title {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--color-text-primary);
  margin-bottom: 2rem;
}

.showcase__section {
  margin-bottom: 2rem;
  padding: 1.5rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  background-color: var(--color-bg-primary);
}

.showcase__section-title {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 1rem;
}

.showcase__hint {
  font-size: 0.8125rem;
  color: var(--color-text-tertiary);
  margin-bottom: 1rem;
}

.showcase__row {
  display: flex;
  align-items: center;
  gap: 1rem;
  flex-wrap: wrap;
}

.showcase__column {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}
</style>
