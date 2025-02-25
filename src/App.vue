<script setup>
import { ref, onMounted, onUnmounted } from "vue"
import { convertFileSrc, invoke } from "@tauri-apps/api/core"
import ImShow from "./ImShow.vue"
import { readFile, BaseDirectory } from '@tauri-apps/plugin-fs'

const imgSrcs = ref([])

const handleKeydown = (event) => {
  if (event.key === 'Escape') {
    invoke('quit_app')
  }
}

const loadImgBlobs = async () => {
  const imgPaths = await invoke('get_image_paths');
  imgSrcs.value = await Promise.all(imgPaths.map(async (path) => {
    return convertFileSrc(path)
  }))
}

onMounted(() => {
  loadImgBlobs()
  window.addEventListener('keydown', handleKeydown)
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

<style scoped></style>
