<template>
  <div>
    <!-- 默认重定向到文章页面 -->
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()
router.replace('/articles')
</script>

