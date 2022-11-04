<template><div></div></template>

<script lang="ts">
import { list } from '@/pages/organizations/OrganizationService';
import { routes } from '@/routes';
import { Vue } from 'vue-class-component';

export default class Root extends Vue {
  async mounted() {
    await list().then(async (organizations) => {
      await this.$router.push({
        name: routes.Home,
        query: {
          organization_id: organizations[0].organization_id,
        },
      });
    });
  }
}
</script>
