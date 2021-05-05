<template>
  <Promised :promise="applications$">
    <!-- Use the "pending" slot to display a loading message -->
    <template #pending>
      <p>Loading...</p>
    </template>
    <!-- The default scoped slot will be used as the result -->
    <template #default="applications">
      <ul>
        <li v-for="application in applications" :key="application.application__id">
          {{ application.application__id }} - {{ application.name }}
        </li>
      </ul>
    </template>
    <!-- The "rejected" scoped slot will be used if there is an error -->
    <template #rejected="error">
      <p>Error: {{ error.message }}</p>
    </template>
  </Promised>
</template>

<script>
import ApplicationService from './ApplicationService';

export default {
  name: "ApplicationsList",
  components: {},
  props: {
    msg: String
  },
  data: () => {
    return {
      applications$: new Promise(() => {
      }),
    }
  },
  created() {
    this.applications$ = ApplicationService.list();
  },
};
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
</style>
