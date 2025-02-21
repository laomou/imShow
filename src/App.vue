<script setup>
import { ref, onMounted, onUnmounted } from "vue"
import { invoke } from "@tauri-apps/api/core"
import ImShow from "./ImShow.vue"
import { readFile, BaseDirectory } from '@tauri-apps/plugin-fs'

const imgBlobs = ref([])

const handleKeydown = (event) => {
  if (event.key === 'Escape') {
    invoke('quit_app')
  }
}

const loadImgBlobs = async () => {
  const imgNames = await invoke('get_image_names');
  imgBlobs.value = await Promise.all(imgNames.map(async (name) => {
    return { blob: await readFile(`${name}`, { baseDir: BaseDirectory.AppCache }), type: name.split('.').pop() }
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
    <ImShow :imgBlobs="imgBlobs"></ImShow>
  </main>
</template>

<style scoped></style>
