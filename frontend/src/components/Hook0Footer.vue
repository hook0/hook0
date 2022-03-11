<template>
  <footer>
    <div class="footer-container">
      <nav class="footer-items" aria-label="Footer">
        <div class="footer-item">
          <hook0-button href="mailto:support@hook0.com">
            Contact us
          </hook0-button>
          -
          <hook0-button href="https://gitlab.com/hook0/hook0/-/issues/new">
            Report issue
          </hook0-button>
        </div>
      </nav>

      <div class="footer-icons">
        <a href="https://twitter.com/hook0_" class="footer-icon">
          <svg class="h-6 w-6" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">

            <path
              d="M23.953 4.57a10 10 0 01-2.825.775 4.958 4.958 0 002.163-2.723c-.951.555-2.005.959-3.127 1.184a4.92 4.92 0 00-8.384 4.482C7.69 8.095 4.067 6.13 1.64 3.162a4.822 4.822 0 00-.666 2.475c0 1.71.87 3.213 2.188 4.096a4.904 4.904 0 01-2.228-.616v.06a4.923 4.923 0 003.946 4.827 4.996 4.996 0 01-2.212.085 4.936 4.936 0 004.604 3.417 9.867 9.867 0 01-6.102 2.105c-.39 0-.779-.023-1.17-.067a13.995 13.995 0 007.557 2.209c9.053 0 13.998-7.496 13.998-13.985 0-.21 0-.42-.015-.63A9.935 9.935 0 0024 4.59z"/>
          </svg>
        </a>

        <a href="https://gitlab.com/hook0" class="footer-icon">
          <span class="sr-only">Gitlab</span>
          <svg class="h-6 w-6" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path
              d="M4.845.904c-.435 0-.82.28-.955.692C2.639 5.449 1.246 9.728.07 13.335a1.437 1.437 0 00.522 1.607l11.071 8.045c.2.145.472.144.67-.004l11.073-8.04a1.436 1.436 0 00.522-1.61c-1.285-3.942-2.683-8.256-3.817-11.746a1.004 1.004 0 00-.957-.684.987.987 0 00-.949.69l-2.405 7.408H8.203l-2.41-7.408a.987.987 0 00-.942-.69h-.006zm-.006 1.42l2.173 6.678H2.675zm14.326 0l2.168 6.678h-4.341zm-10.593 7.81h6.862c-1.142 3.52-2.288 7.04-3.434 10.559L8.572 10.135zm-5.514.005h4.321l3.086 9.5zm13.567 0h4.325c-2.467 3.17-4.95 6.328-7.411 9.502 1.028-3.167 2.059-6.334 3.086-9.502zM2.1 10.762l6.977 8.947-7.817-5.682a.305.305 0 01-.112-.341zm19.798 0l.952 2.922a.305.305 0 01-.11.341v.002l-7.82 5.68.026-.035z"/>
          </svg>
        </a>

        <a href="https://github.com/hook0" class="footer-icon">
          <span class="sr-only">GitHub</span>
          <svg class="h-6 w-6" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path
              d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"/>
          </svg>
        </a>
      </div>
      <p class="footer-text">
        Open-source always win in the end.
      </p>
    </div>
  </footer>
</template>

<script lang="ts">

export default {
  name: 'hook0-footer',
  components: {},
  data() {
    return {};
  },
};
</script>

<style lang="scss" scoped>
.footer-container {
  @apply max-w-7xl mx-auto py-12 px-4 overflow-hidden sm:px-6 lg:px-8;
}

.footer-icons {
  @apply mt-8 flex justify-center space-x-6;
}

.footer-icon {
  @apply text-gray-400 hover:text-gray-500;
}

.footer-text {
  @apply mt-8 text-center text-base text-gray-400;
}

.footer-items {
  @apply -mx-5 -my-2 flex flex-wrap justify-center;
}

.footer-item {
  @apply px-5 py-2;
}
</style>
