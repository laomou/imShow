<script setup>
import { ref, onMounted, onUpdated, onUnmounted, watch } from "vue"
import * as PIXI from "pixi.js"
import { Button, CheckBox } from '@pixi/ui'
import exifr from 'exifr'
import { convertFileSrc, invoke } from "@tauri-apps/api/core"
import { warn, debug, trace, info, error } from '@tauri-apps/plugin-log'
import rotateLeftIcon from '@/assets/rotate-left.svg'
import resetIcon from '@/assets/reset.svg'
import leftCmpIcon from '@/assets/left-cmp.svg'
import rotateRightIcon from '@/assets/rotate-right.svg'
import flipHorizontalIcon from '@/assets/flip-horizontal.svg'
import switchOffIcon from '@/assets/switch-off.svg'
import switchOnIcon from '@/assets/switch-on.svg'

const pixiContainer = ref(null)
let pixi_app = null
let blockRects = []
let blockViews = []
let selectedBlockIndex = -1
let isDragging = false
let dragStart = { x: 0, y: 0 }
let animationFrameId = null
let histogramCache = new Map()

const isMouseInBlock = (x, y, blockRect) => {
  return x >= blockRect.x && x <= blockRect.x + blockRect.width && y >= blockRect.y && y <= blockRect.y + blockRect.height
}

const toggleExif = () => {
  blockViews.forEach((blockView) => {
    blockView.toggleExif()
  })
}

const toggleHistogram = () => {
  blockViews.forEach((blockView) => {
    blockView.toggleHistogram()
  })
}

const toggleCenterMark = () => {
  blockViews.forEach((blockView) => {
    blockView.toggleCenterMark()
  })
}

const handleFlipHorizontal = () => {
  if (selectedBlockIndex != -1) {
    blockViews[selectedBlockIndex].flip_h()
  } else {
    blockViews.forEach((blockView) => {
      blockView.flip_h()
    })
  }
}

const handleRotateLeft = () => {
  if (selectedBlockIndex != -1) {
    blockViews[selectedBlockIndex].rotate(-90)
  } else {
    blockViews.forEach((blockView) => {
      blockView.rotate(-90)
    })
  }
}

const handleCmpDown = () => {
  if (selectedBlockIndex != -1) {
    blockViews[0].update(blockViews[selectedBlockIndex].sprite)
  } else {
    if (blockViews.length == 2) {
      blockViews[0].update(blockViews[1].sprite)
    }
  }
}

const handleCmpUp = () => {
  if (selectedBlockIndex != -1) {
    blockViews[0].restore()
  } else {
    if (blockViews.length == 2) {
      blockViews[0].restore()
    }
  }
}

const handleReset = () => {
  if (selectedBlockIndex != -1) {
    blockViews[selectedBlockIndex].reset()
    selectedBlockIndex = -1
  } else {
    blockViews.forEach((blockView) => {
      blockView.reset()
    })
  }
}

const handleRotateRight = () => {
  if (selectedBlockIndex != -1) {
    blockViews[selectedBlockIndex].rotate(90)
  } else {
    blockViews.forEach((blockView) => {
      blockView.rotate(90)
    })
  }
}

const handleDoubleClick = (event) => {
  if (blockViews.length < 2) return

  const index = blockRects.findIndex((blockRect) => isMouseInBlock(event.clientX, event.clientY, blockRect))
  if (index < 1 || index >= blockViews.length) return

  if (selectedBlockIndex === index) {
    blockViews[index].toggleBorder()
    selectedBlockIndex = -1
  } else {
    if (selectedBlockIndex !== -1) {
      blockViews[selectedBlockIndex].toggleBorder()
    }
    selectedBlockIndex = index
    blockViews[index].toggleBorder()
  }
}


const handleMouseDown = (event) => {
  dragStart = { x: event.clientX, y: event.clientY }
  isDragging = true
}

const handleMouseMove = (event) => {
  if (isDragging) {
    if (animationFrameId) {
      cancelAnimationFrame(animationFrameId)
    }
    animationFrameId = requestAnimationFrame(() => {
      const dragEnd = { x: event.clientX, y: event.clientY }
      const offsetX = dragEnd.x - dragStart.x
      const offsetY = dragEnd.y - dragStart.y

      blockViews.forEach((blockView) => {
        blockView.move(offsetX, offsetY)
      })

      dragStart = dragEnd
      animationFrameId = null
    })
  }
}

const handleMouseUp = () => {
  isDragging = false
}

const handleWheel = (event, scaleFactor = 1.1) => {
  event.preventDefault()

  const delta = event.deltaY > 0 ? -1 : 1
  const scaleChange = delta > 0 ? scaleFactor : 1 / scaleFactor

  if (event.ctrlKey) {
    const index = blockRects.findIndex((blockRect) =>
      isMouseInBlock(event.clientX, event.clientY, blockRect)
    )

    if (index !== -1) {
      blockViews[index].scale(scaleChange)
    }
  } else {
    blockViews.forEach(blockView => {
      blockView.scale(scaleChange)
    })
  }
}

const handleKeydown = (event) => {
  if (event.key === 'Escape') {
    invoke('quit_app')
  }
}

const Colors = {
  BACKGROUND: 0xffffff,
  EXIF_TEXT: 0x00ff00,
  HIST_R: 0xff0000,
  HIST_G: 0x00ff00,
  HIST_B: 0x0000ff,
  HIST_GRAY: 0xcccccc,
  HIST_LABELS: 0xcccccc
}

class Toolbar {
  constructor(pixi_app) {
    this.container = new PIXI.Container()
    this.container.x = 0
    this.container.y = 0
    this.container.zIndex = 90
    this.container.visible = false
    pixi_app.stage.addChild(this.container)

    this.viewWidth = pixi_app.screen.width
    this.centerX = this.viewWidth / 2
    this.buttons = []
    this.checkboxs = []
  }

  async initLayout(hasCmp = false) {
    PIXI.Assets.add({ alias: 'rotate-left', src: rotateLeftIcon })
    PIXI.Assets.add({ alias: 'reset', src: resetIcon })
    PIXI.Assets.add({ alias: 'left-cmp', src: leftCmpIcon })
    PIXI.Assets.add({ alias: 'rotate-right', src: rotateRightIcon })
    PIXI.Assets.add({ alias: 'flip-h', src: flipHorizontalIcon })
    PIXI.Assets.add({ alias: 'switch-off', src: switchOffIcon })
    PIXI.Assets.add({ alias: 'switch-on', src: switchOnIcon })

    const btnTextures = await PIXI.Assets.load(['rotate-left', 'left-cmp', 'reset', 'rotate-right', 'flip-h'])
    this.addButton(btnTextures['rotate-left'], () => {
      handleRotateLeft()
    })
    if (hasCmp) {
      this.addButton(btnTextures['left-cmp'], () => {
      }, () => {
        handleCmpDown()
      }, () => {
        handleCmpUp()
      })
    }
    this.addButton(btnTextures['reset'], () => {
      handleReset()
    })
    this.addButton(btnTextures['rotate-right'], () => {
      handleRotateRight()
    })
    this.addButton(btnTextures['flip-h'], () => {
      handleFlipHorizontal()
    })
    const cbxTextures = await PIXI.Assets.load(['switch-off', 'switch-on'])
    this.addCheckBox('Exif', cbxTextures['switch-off'], cbxTextures['switch-on'], (checked) => {
      toggleExif()
    })
    this.addCheckBox('Hist', cbxTextures['switch-off'], cbxTextures['switch-on'], (checked) => {
      toggleHistogram()
    })
    this.addCheckBox('+', cbxTextures['switch-off'], cbxTextures['switch-on'], (checked) => {
      toggleCenterMark()
    })
    this.updatePositions()

    pixi_app.render()
  }

  addButton(texture, onPress, onDown = null, onUp = null) {
    const sprite = new PIXI.Sprite(texture)
    const button = new Button(sprite)
    button.x = 0
    button.y = 0
    button.width = texture.width
    button.height = texture.height
    this.container.addChild(button.view)
    this.buttons.push(button)

    onPress && button.onPress.connect(onPress)
    onDown && button.onDown.connect(onDown)
    onUp && button.onUp && button.onUp.connect(onUp)
  }

  addCheckBox(label, uncheckedTexture, checkedTexture, onChange) {
    const checkbox = new CheckBox({
      text: label,
      style: {
        unchecked: uncheckedTexture,
        checked: checkedTexture
      }
    })
    checkbox.x = 0
    checkbox.y = 0
    this.container.addChild(checkbox)
    this.checkboxs.push(checkbox)

    onChange && checkbox.onChange.connect(onChange)
  }

  updatePositions(spacing = 8) {
    const totalButtonWidth = this.buttons.reduce((sum, button) => sum + button.width + spacing, 0) - spacing
    const buttonStartX = this.centerX - totalButtonWidth / 2
    this.buttons.forEach((button, index) => {
      button.view.x = buttonStartX + index * (button.width + spacing)
      button.view.y = 0
    })
    const totalCheckboxWidth = this.checkboxs.reduce((sum, checkbox) => {
      const iconWidth = checkbox.width
      const textWidth = checkbox.label ? checkbox.label.width : 0
      return sum + iconWidth + textWidth + spacing
    }, 0) - spacing
    const checkboxStartX = this.viewWidth - totalCheckboxWidth
    let currentX = checkboxStartX
    this.checkboxs.forEach((checkbox) => {
      checkbox.x = currentX
      checkbox.y = 0
      const iconWidth = checkbox.width
      const textWidth = checkbox.label ? checkbox.label.width : 0
      currentX += iconWidth + textWidth + spacing
    })

    this.container.visible = true
  }
}

class Histogram {
  constructor(texture, width = 256, height = 100) {
    this.texture = texture
    this.mode = "rgb"
    this.viewWidth = width
    this.viewHeight = height

    this.view = new PIXI.Container()
    this.view.x = 0
    this.view.y = 0
    this.view.zIndex = 100
    this.hitArea = new PIXI.Rectangle(0, 0, this.viewWidth, this.viewHeight)
    this.view.hitArea = this.hitArea
    this.view.visible = false
    this.view.interactive = true
    this.view.on('pointerdown', this.toggleMode, this)

    this.histogramGraphics = new PIXI.Graphics()
    this.view.addChild(this.histogramGraphics)

    const labelPoints = [0, 64, 128, 192, 255]
    labelPoints.forEach(value => {
      const text = new PIXI.Text(value.toString(), {
        fontSize: 10,
        fill: Colors.HIST_LABELS,
        align: 'center'
      })
      text.x = value - text.width / 2 + 4
      text.y = 105
      this.view.addChild(text)
    })
    this.update(this.texture)
  }

  toggleMode() {
    this.mode = this.mode === 'rgb' ? 'gray' : 'rgb'
    this.update(this.texture)

    pixi_app.render()
  }

  toggleView() {
    this.view.visible = !this.view.visible
  }

  update(texture) {
    const textureId = texture.baseTexture.uid
    if (histogramCache.has(textureId)) {
      this.drawHistogram(histogramCache.get(textureId))
      return
    }
    const canvas = texture.source.resource
    const context = canvas.getContext('2d', { willReadFrequently: true })
    const imgData = context.getImageData(0, 0, canvas.width, canvas.height)
    const histogram = this.calculateHistogram(imgData)
    histogramCache.set(textureId, histogram)
    this.drawHistogram(histogram)
  }

  calculateHistogram(imgData) {
    const histogram = {
      r: new Array(256).fill(0),
      g: new Array(256).fill(0),
      b: new Array(256).fill(0),
      gray: new Array(256).fill(0)
    }

    for (let i = 0; i < imgData.data.length; i += 4) {
      const r = imgData.data[i]
      const g = imgData.data[i + 1]
      const b = imgData.data[i + 2]
      const gray = Math.round(0.299 * r + 0.587 * g + 0.114 * b)

      histogram.r[r]++
      histogram.g[g]++
      histogram.b[b]++
      histogram.gray[gray]++
    }

    return histogram
  }

  drawHistogram(histogram) {
    this.histogramGraphics.clear()
    const max = Math.max(...Object.values(histogram).flat())

    const drawChannel = (channel, color, offset) => {
      this.histogramGraphics.lineStyle(1, color)
      this.histogramGraphics.beginFill(color, 0.5)
      for (let i = 0; i < 256; i++) {
        const height = (histogram[channel][i] / max) * this.viewHeight
        this.histogramGraphics.drawRect(i + offset, this.viewHeight - height, 1, height)
      }
      this.histogramGraphics.endFill()
    }

    if (this.mode === 'rgb') {
      drawChannel('r', Colors.HIST_R, 4)
      drawChannel('g', Colors.HIST_G, 4)
      drawChannel('b', Colors.HIST_B, 4)
    } else {
      drawChannel('gray', Colors.HIST_GRAY, 4)
    }
  }
}

class ExifText {
  constructor(image) {
    this.image = image
    this.view = new PIXI.Container()
    this.view.x = 0
    this.view.y = 0
    this.view.visible = false
    this.view.zIndex = 100

    this.exif = new PIXI.Text('', { fill: Colors.EXIF_TEXT, fontSize: 16 })
    this.exif.x = 5
    this.exif.y = 130
    this.view.addChild(this.exif)

    this.readExif(this.image)
  }

  toggleView() {
    this.view.visible = !this.view.visible
  }

  async readExif(img) {
    let exifInfo = ''
    const fileName = decodeURIComponent(img.src).split(/[\/\\]/).pop()
    exifInfo += `${fileName}, ${img.width}x${img.height}\n`
    exifInfo += '------------\n'
    try {
      const exifData = await exifr.parse(img, { icc: true })
      if (exifData) {
        if (exifData.ExposureTime) {
          const exposureTime = exifData.ExposureTime < 1
            ? `1/${Math.round(1 / exifData.ExposureTime)}`
            : exifData.ExposureTime
          exifInfo += `${exposureTime} s\n`
        }
        if (exifData.FNumber) exifInfo += `f/${exifData.FNumber}\n`
        if (exifData.ISO) exifInfo += `ISO ${exifData.ISO}\n`
        if (exifData.FocalLength) exifInfo += `${Math.round(exifData.FocalLength)} mm\n`
        exifInfo += '\n'
        if (exifData.Model) exifInfo += `${exifData.Model}\n`
        if (exifData.Make) exifInfo += `${exifData.Make}\n`
        exifInfo += '\n'
        if (exifData.ColorSpaceData) exifInfo += `${exifData.ColorSpaceData}\n`
        if (exifData.ProfileDescription) exifInfo += `${exifData.ProfileDescription}\n`
      } else {
        exifInfo += 'No EXIF data found'
      }
    } catch (err) {
      error(`exifr.parse ${err}`)
      exifInfo += 'No EXIF data found'
    }

    this.drawExifText(exifInfo)
  }

  drawExifText(text) {
    this.exif.text = text
  }
}

class Viewport {
  constructor(imgSrc, viewRect) {
    this.imgSrc = imgSrc
    this.viewRect = viewRect
    this.initScale = 1
    this.centerX = this.viewRect.width / 2
    this.centerY = this.viewRect.height / 2

    this.view = new PIXI.Container()
    this.view.x = viewRect.x
    this.view.y = viewRect.y

    const mask = new PIXI.Graphics()
    mask.beginFill(Colors.BACKGROUND)
    mask.drawRect(0, 0, viewRect.width, viewRect.height)
    mask.endFill()
    this.view.addChild(mask)
    this.view.mask = mask

    this.border = new PIXI.Graphics()
    this.border.beginFill(Colors.BACKGROUND)
    this.border.lineStyle(2, 0x00ff00, 1)
    this.border.drawRect(0, 0, viewRect.width, viewRect.height)
    this.border.endFill()
    this.border.visible = false
    this.view.addChild(this.border)

    this.centerMark = new PIXI.Graphics()
    this.centerMark.beginFill(Colors.BACKGROUND)
    this.centerMark.lineStyle(1, 0xff0000, 1)
    this.centerMark.moveTo(this.viewRect.width / 2, this.viewRect.height / 2 - 10)
    this.centerMark.lineTo(this.viewRect.width / 2, this.viewRect.height / 2 + 10)
    this.centerMark.moveTo(this.viewRect.width / 2 - 10, this.viewRect.height / 2)
    this.centerMark.lineTo(this.viewRect.width / 2 + 10, this.viewRect.height / 2)
    this.centerMark.endFill()
    this.centerMark.visible = false
    this.centerMark.zIndex = 100
    this.view.addChild(this.centerMark)
  }

  flip_h() {
    const angle = ((this.sprite.angle % 360) + 360) % 360
    const isVertical = angle === 90 || angle === 270 || angle === -90

    if (isVertical) {
      this.sprite.scale.y *= -1
    } else {
      this.sprite.scale.x *= -1
    }

    pixi_app.render()
  }

  rotate(angle) {
    this.sprite.angle = (this.sprite.angle + angle + 360) % 360

    pixi_app.render()
  }

  move(offsetX, offsetY) {
    this.sprite.x += offsetX
    this.sprite.y += offsetY

    pixi_app.render()
  }

  scale(scale) {
    const newScaleX = this.sprite.scale.x * scale
    const newScaleY = this.sprite.scale.y * scale

    if (newScaleX >= this.initScale || newScaleY >= this.initScale) {
      this.sprite.x = this.centerX + (this.sprite.x - this.centerX) * scale
      this.sprite.y = this.centerY + (this.sprite.y - this.centerY) * scale
      this.sprite.scale.set(newScaleX, newScaleY)

      pixi_app.render()
    }
  }

  reset() {
    this.sprite.x = this.viewRect.width / 2
    this.sprite.y = this.viewRect.height / 2
    this.sprite.scale.set(this.initScale, this.initScale)
    this.sprite.angle = 0
    this.border.visible = false

    pixi_app.render()
  }

  toggleBorder() {
    this.border.visible = !this.border.visible

    pixi_app.render()
  }
  
  toggleCenterMark() {
    this.centerMark.visible = !this.centerMark.visible

    pixi_app.render()
  }

  toggleExif() {
    this.exif.toggleView()

    pixi_app.render()
  }

  toggleHistogram() {
    this.histogram.toggleView()

    pixi_app.render()
  }

  update(sprite) {
    this.sprite.visible = false
    this.tmpSprite = new PIXI.Sprite(sprite.texture)
    if (sprite.filters) {
      this.tmpSprite.filters = sprite.filters
    }
    this.tmpSprite.scale.set(sprite.scale.x, sprite.scale.y)
    this.tmpSprite.angle = sprite.angle
    this.tmpSprite.x = sprite.x
    this.tmpSprite.y = sprite.y
    this.tmpSprite.width = sprite.width
    this.tmpSprite.height = sprite.height
    this.tmpSprite.anchor.set(0.5)
    this.view.addChild(this.tmpSprite)
    this.histogram.update(sprite.texture)

    pixi_app.render()
  }

  restore() {
    this.view.removeChild(this.tmpSprite)
    this.sprite.visible = true
    this.histogram.update(this.sprite.texture)

    pixi_app.render()
  }

  initLayout() {
    return new Promise((resolve, reject) => {
      const img = new Image()
      img.crossOrigin = "anonymous"
      img.src = this.imgSrc
      img.onload = async () => {
        const texture = PIXI.Texture.from(img)
        this.sprite = new PIXI.Sprite(texture)
        this.sprite.x = this.viewRect.width / 2
        this.sprite.y = this.viewRect.height / 2
        this.initScale = Math.min(this.viewRect.width / texture.width, this.viewRect.height / texture.height)
        this.sprite.scale.set(this.initScale, this.initScale)
        this.sprite.anchor.set(0.5, 0.5)
        this.sprite.angle = 0
        this.view.addChild(this.sprite)

        await Promise.all([
          new Promise(resolve => {
            this.histogram = new Histogram(texture)
            this.view.addChild(this.histogram.view)
            resolve()
          }),
          new Promise(resolve => {
            this.exif = new ExifText(img)
            this.view.addChild(this.exif.view)
            resolve()
          }),
        ])

        pixi_app.render()
        resolve()
      }
      img.onerror = reject
    })
  }
}

function calculateBlockRects(canvasWidth, canvasHeight, numBlocks, paddingX = 4, paddingY = 4, paddingTop = 25) {
  const blocks = []
  let cols = Math.ceil(Math.sqrt(numBlocks))
  let rows = Math.ceil(numBlocks / cols)
  // hook for 3 blocks
  if (numBlocks == 3) {
    cols = 3
    rows = 1
  }
  const blockWidth = (canvasWidth - (cols - 1) * paddingX) / cols
  const blockHeight = (canvasHeight - (rows - 1) * paddingY) / rows

  for (let i = 0; i < numBlocks; i++) {
    const col = i % cols
    const row = Math.floor(i / cols)
    const x = col * (blockWidth + paddingX)
    const y = row * (blockHeight + paddingY) + paddingTop
    blocks.push({ x, y, width: blockWidth, height: blockHeight })
  }

  return blocks
}

const initLayout = async (imgSrcs) => {
  const toolbar = new Toolbar(pixi_app)
  await toolbar.initLayout(imgSrcs.length > 1)
  blockRects = calculateBlockRects(pixi_app.canvas.width, pixi_app.canvas.height, imgSrcs.length)
  blockViews = blockRects.map((rect, index) => {
    const viewport = new Viewport(imgSrcs[index], rect)
    pixi_app.stage.addChild(viewport.view)
    return viewport
  })
  await Promise.all(blockViews.map((viewport) => {
    viewport.initLayout()
  }))
}

const initPIXIApp = async () => {
  pixi_app = new PIXI.Application()
  await pixi_app.init({ background: Colors.BACKGROUND, resizeTo: window, preference: 'webgl', autoStart: false })
  pixiContainer.value.appendChild(pixi_app.canvas)

  pixi_app.canvas.oncontextmenu = (event) => false
  pixi_app.canvas.addEventListener('wheel', handleWheel, { passive: false })
  pixi_app.canvas.addEventListener('dblclick', handleDoubleClick)
  pixi_app.canvas.addEventListener('mousedown', handleMouseDown)
  pixi_app.canvas.addEventListener('mousemove', handleMouseMove)
  pixi_app.canvas.addEventListener('mouseup', handleMouseUp)
}

onMounted(async () => {
  await initPIXIApp()
  const imgPaths = await invoke('get_image_paths')
  const imgSrcs = imgPaths.map((path) => {
    return convertFileSrc(path)
  })
  initLayout(imgSrcs).catch(err => {
    error('Failed to initialize layout:', err)
  })
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  if (pixi_app) {
    pixi_app.canvas.removeEventListener('wheel', handleWheel, { passive: false })
    pixi_app.canvas.removeEventListener('dblclick', handleDoubleClick)
    pixi_app.canvas.removeEventListener('mousedown', handleMouseDown)
    pixi_app.canvas.removeEventListener('mousemove', handleMouseMove)
    pixi_app.canvas.removeEventListener('mouseup', handleMouseUp)
    pixi_app.destroy()
    pixi_app = null
  }
  if (animationFrameId) {
    cancelAnimationFrame(animationFrameId)
    animationFrameId = null
  }
  window.removeEventListener('keydown', handleKeydown)
})

</script>

<template>
  <div ref="pixiContainer"></div>
</template>

<style scoped>
.pixi-container {
  width: 100vw;
  height: 100vh;
}
</style>
