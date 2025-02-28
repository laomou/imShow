<script setup>
import { ref, onMounted, onUnmounted } from "vue"
import { convertFileSrc, invoke } from "@tauri-apps/api/core"
import ImShow from "./ImShow.vue"

const imgSrcs = ref([])

const handleKeydown = (event) => {
  if (event.key === 'Escape') {
    invoke('quit_app')
  }
}

const loadImgPaths = async () => {
  const imgPaths = await invoke('get_image_paths');
  imgSrcs.value = await Promise.all(imgPaths.map(async (path) => {
    return convertFileSrc(path)
  }))
}

onMounted(async () => {
  window.addEventListener('keydown', handleKeydown)
  await loadImgPaths()
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})

</script>

<template>
  <main class="container">
    <ImShow :imgSrcs="imgSrcs"></ImShow>
  </main>
</template>

<style scoped>
.container {
  width: 100%;
  height: 100vh;
  overflow: hidden;
}
</style>
