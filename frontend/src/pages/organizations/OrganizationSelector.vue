<template>
  <Promised :promise="organizations$">
    <!-- Use the "pending" slot to display a loading message -->
    <template #pending>
      <p>Loading...</p>
    </template>

    <!-- The default scoped slot will be used as the result -->
    <template #default="organizations">
      <select id="organizations_select" :value="current_organization.organization__id" @change="set_current_organization_by_id($event.target.value)">
        <option v-for="organization in organizations"
                :key="organization.organization_id"
                :value="organization.organization__id"
        >{{ organization.organization__id }}
        </option>
      </select>
    </template>

    <!-- The "rejected" scoped slot will be used if there is an error -->
    <template #rejected="error">
      <p>Error: {{ error.message }}</p>
    </template>
  </Promised>
</template>

<script lang="ts">
import OrganizationService, {Organization} from './OrganizationService';
import {Options, Vue} from "vue-class-component";
import {UUID} from "@/http";

export default class OrganizationSelector extends Vue {

  private organizations$!: Promise<Array<Organization>>;
  private current_organization = OrganizationService.current_organization;

  set_current_organization_by_id(organization_id: UUID): void {
    this.organizations$.then(organizations => {
      const new_organization = organizations.find(org => org.organization__id === organization_id);
      // if no organization where found fallback on current value
      OrganizationService.current_organization.value = new_organization || OrganizationService.current_organization;
    });
  }


  created(): void {
    this.organizations$ = OrganizationService.list();
    this.organizations$.then((organizations) => OrganizationService.current_organization.value = organizations[0]);
  }

};
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
select{
  width:100%;
}
</style>
