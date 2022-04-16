<template>
  <hook0-card>
    <hook0-card-header>
      <template #header>Hello 👋</template>
      <template #subtitle>Hook0 API is complete and production ready but the UI is still a work in progress...
      </template>
    </hook0-card-header>
    <hook0-card-content>
      <hook0-card-content-lines>
        <hook0-card-content-line type="full-width">
          <template #content>
            <hook0-text>Start your journey by selecting your
              <hook0-text class="bold">organization</hook0-text>
              on the left selector and then creating your first application.
            </hook0-text>
          </template>
        </hook0-card-content-line>
      </hook0-card-content-lines>
    </hook0-card-content>

  </hook0-card>
</template>
<script lang="ts">
import {Vue, Options} from 'vue-class-component';
import {routes} from "@/routes";
import Hook0CardContent from "@/components/Hook0CardContent.vue";

@Options({
  components: {Hook0CardContent}
})
export default class Home extends Vue {
  mounted() {
    // do nothing, let the selector do its job
  }
};

</script>
