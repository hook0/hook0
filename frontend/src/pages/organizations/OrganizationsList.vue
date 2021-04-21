<template>
  <Promised :promise="organizations$">
    <!-- Use the "pending" slot to display a loading message -->
    <template #pending>
      <p>Loading...</p>
    </template>
    <!-- The default scoped slot will be used as the result -->
    <template #default="organizations">
      <ul>
        <li v-for="organization in organizations" :key="organization.organization__id">
          {{ organization.organization__id }} - {{ organization.role }}
        </li>
      </ul>
    </template>
    <!-- The "rejected" scoped slot will be used if there is an error -->
    <template #rejected="error">
      <p>Error: {{ error.message }}</p>
    </template>
  </Promised>
</template>

<script lang="ts">
import OrganizationService, {Organization} from './OrganizationService';
import {Options, Vue} from 'vue-class-component';


export default class OrganizationList extends Vue {
  private organizations$: Promise<Array<Organization>> = OrganizationService.list();


  created(): void {
  }
};
</script>
