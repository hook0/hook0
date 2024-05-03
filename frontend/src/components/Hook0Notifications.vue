<script setup lang="ts">
import { Notivue, Notification, NotificationProgress } from 'notivue';
</script>

<template>
  <Notivue v-slot="item">
    <Notification :item="item" />
    <NotificationProgress :item="item" />
  </Notivue>
</template>
