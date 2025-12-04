<template>
  <div class="zhulong-container">
    <NuxtPage />
  </div>
</template>

<script setup lang="ts">
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #__nuxt {
  height: 100%;
  width: 100%;
  overflow: hidden;
  background-color: #ffffff;
  color: #000000;
}

.zhulong-container {
  height: 100%;
  width: 100%;
  display: flex;
  flex-direction: column;
  background-color: #ffffff;
  color: #000000;
}
</style>

