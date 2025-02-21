<script setup>
import { ref, computed, onMounted, onUpdated, onUnmounted, watch } from "vue"
import * as PIXI from "pixi.js"
import { Button } from '@pixi/ui'

const props = defineProps({
    imgBlobs: {
        type: Array,
        required: true
    }
})

const pixiContainer = ref(null)
let app = null
let blockRects = []
let blockViews = []
let selectedBlockIndex = -1
let tmpCmpSprite = null
let isDragging = false
let dragStart = { x: 0, y: 0 }

const isMouseInBlock = (x, y, blockRect) => {
    return x >= blockRect.x && x <= blockRect.x + blockRect.width && y >= blockRect.y && y <= blockRect.y + blockRect.height
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
        blockViews[0].sprite.visible = false
        if (tmpCmpSprite) {
            tmpCmpSprite.destroy()
        }
        tmpCmpSprite = new PIXI.Sprite(blockViews[selectedBlockIndex].sprite.texture)
        tmpCmpSprite.scale.set(blockViews[selectedBlockIndex].sprite.scale.x, blockViews[selectedBlockIndex].sprite.scale.y)
        tmpCmpSprite.angle = blockViews[selectedBlockIndex].sprite.angle
        tmpCmpSprite.x = blockViews[selectedBlockIndex].sprite.x
        tmpCmpSprite.y = blockViews[selectedBlockIndex].sprite.y
        tmpCmpSprite.width = blockViews[selectedBlockIndex].sprite.width
        tmpCmpSprite.height = blockViews[selectedBlockIndex].sprite.height
        tmpCmpSprite.anchor.set(0.5)
        blockViews[0].container.addChild(tmpCmpSprite)
    }
}

const handleCmpUp = () => {
    if (selectedBlockIndex != -1) {
        blockViews[0].container.removeChild(tmpCmpSprite)
        blockViews[0].sprite.visible = true
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
    if (blockViews.length < 2) {
        return
    }
    const index = blockRects.findIndex((blockRect) => isMouseInBlock(event.clientX, event.clientY, blockRect))
    if (index >= 1) {
        if (selectedBlockIndex === index) {
            blockViews[index].toggleBorder()
            selectedBlockIndex = -1
        } else {
            if (selectedBlockIndex != -1) {
                blockViews[selectedBlockIndex].toggleBorder()
            }
            selectedBlockIndex = index
            blockViews[index].toggleBorder()
        }
    }
}


const handleMouseDown = (event) => {
    dragStart = { x: event.clientX, y: event.clientY }
    isDragging = true
}

const handleMouseMove = (event) => {
    if (isDragging) {
        const dragEnd = { x: event.clientX, y: event.clientY }
        const offsetX = dragEnd.x - dragStart.x
        const offsetY = dragEnd.y - dragStart.y

        blockViews.forEach((blockView) => {
            blockView.move(offsetX, offsetY)
        })

        dragStart = dragEnd
    }
}

const handleMouseUp = () => {
    isDragging = false
}

const handleWheel = (event, scaleFactor = 1.1) => {
    const delta = event.deltaY > 0 ? -1 : 1
    const scaleChange = delta > 0 ? scaleFactor : 1 / scaleFactor

    let wheelBlockIndex = -1
    if (event.ctrlKey) {
        let index = blockRects.findIndex((blockRect) => isMouseInBlock(event.clientX, event.clientY, blockRect))
        if (index != -1) {
            wheelBlockIndex = index
        }
    }

    if (wheelBlockIndex != -1) {
        const newScaleX = blockViews[wheelBlockIndex].sprite.scale.x * scaleChange
        const newScaleY = blockViews[wheelBlockIndex].sprite.scale.y * scaleChange
        blockViews[wheelBlockIndex].sprite.scale.set(newScaleX, newScaleY)
    } else {
        blockViews.forEach((blockView) => {
            const newScaleX = blockView.sprite.scale.x * scaleChange
            const newScaleY = blockView.sprite.scale.y * scaleChange
            blockView.sprite.scale.set(newScaleX, newScaleY)
        })
    }
}

class Toolbar {
    constructor(app, spacing = 4) {
        this.container = new PIXI.Container()
        this.container.x = 0
        this.container.y = 0
        this.container.zIndex = 90
        app.stage.addChild(this.container)

        this.viewWidth = app.canvas.width
        this.spacing = spacing
        this.realWidth = 0
    }

    async initLayout(hasCmp = false) {
        PIXI.Assets.add({ alias: 'rotate-left', src: 'src/assets/rotate-left.png' })
        PIXI.Assets.add({ alias: 'reset', src: 'src/assets/reset.png' })
        PIXI.Assets.add({ alias: 'left-cmp', src: 'src/assets/left-cmp.png' })
        PIXI.Assets.add({ alias: 'rotate-right', src: 'src/assets/rotate-right.png' })

        const textures = await PIXI.Assets.load(['rotate-left', 'left-cmp', 'reset', 'rotate-right'])
        this.addButton(textures['rotate-left'], () => {
            handleRotateLeft()
        })
        if (hasCmp) {
            this.addButton(textures['left-cmp'], () => {
            }, () => {
                handleCmpDown()
            }, () => {
                handleCmpUp()
            })
        }
        this.addButton(textures['reset'], () => {
            handleReset()
        })
        this.addButton(textures['rotate-right'], () => {
            handleRotateRight()
        })
    }

    addButton(texture, onPress, onDown = null, onUp = null, width = 25, height = 25) {
        const sprite = new PIXI.Sprite(texture)
        const button = new Button(sprite)
        button.view.width = width
        button.view.height = height
        button.view.x = this.realWidth
        this.container.addChild(button.view)

        this.realWidth += width + this.spacing

        onPress && button.onPress.connect(onPress)
        onDown && button.onDown.connect(onDown)
        onUp && button.onUp.connect(onUp)

        this.updatePosition()
    }

    updatePosition() {
        this.container.x = (this.viewWidth - this.realWidth + this.spacing) / 2
    }
}

class Histogram {
    constructor(rootview, imgData, width = 256, height = 100) {
        this.imgData = imgData
        this.mode = "rgb"
        this.viewWidth = width
        this.viewHeight = height

        this.container = new PIXI.Container()
        this.container.x = 0
        this.container.y = 0
        this.container.zIndex = 100
        this.hitArea = new PIXI.Rectangle(0, 0, this.viewWidth, this.viewHeight)
        this.container.hitArea = this.hitArea
        this.container.interactive = true
        this.container.on('pointerdown', this.toggleMode, this)
        rootview.addChild(this.container)
    }

    toggleMode() {
        this.mode = this.mode === 'rgb' ? 'gray' : 'rgb'
        this.update()
    }

    initLayout() {
        this.histogramGraphics = new PIXI.Graphics()
        this.container.addChild(this.histogramGraphics)
        this.update()
    }

    update() {
        const histogram = {
            r: new Array(256).fill(0),
            g: new Array(256).fill(0),
            b: new Array(256).fill(0),
            gray: new Array(256).fill(0)
        }

        for (let i = 0; i < this.imgData.data.length; i += 4) {
            const r = this.imgData.data[i]
            const g = this.imgData.data[i + 1]
            const b = this.imgData.data[i + 2]
            const gray = Math.round(0.299 * r + 0.587 * g + 0.114 * b)

            histogram.r[r]++
            histogram.g[g]++
            histogram.b[b]++
            histogram.gray[gray]++
        }

        this.drawHistogram(histogram)
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
            drawChannel('r', 0xff0000, 0)
            drawChannel('g', 0x00ff00, 0)
            drawChannel('b', 0x0000ff, 0)
        } else {
            drawChannel('gray', 0x888888, 0)
        }
    }
}

class Viewport {
    constructor(app, imgBlob, viewRect) {
        this.imgBlob = imgBlob
        this.viewRect = viewRect
        this.initScale = 1

        this.container = new PIXI.Container()
        this.container.x = viewRect.x
        this.container.y = viewRect.y
        app.stage.addChild(this.container)

        const mask = new PIXI.Graphics()
        mask.beginFill(0xffffff)
        mask.drawRect(0, 0, viewRect.width, viewRect.height)
        mask.endFill()
        this.container.addChild(mask)
        this.container.mask = mask

        this.border = new PIXI.Graphics()
        this.border.beginFill(0xffffff)
        this.border.lineStyle(2, 0x00ff00, 1)
        this.border.drawRect(0, 0, viewRect.width, viewRect.height)
        this.border.endFill()
        this.border.visible = false
        this.container.addChild(this.border)
    }

    rotate(angle) {
        this.sprite.angle = (this.sprite.angle + angle + 360) % 360;
    }

    move(offsetX, offsetY) {
        this.sprite.x += offsetX
        this.sprite.y += offsetY
    }

    reset() {
        this.sprite.x = this.viewRect.width / 2
        this.sprite.y = this.viewRect.height / 2
        this.sprite.scale.set(this.initScale, this.initScale)
        this.sprite.angle = 0
        this.border.visible = false
    }

    toggleBorder() {
        this.border.visible = !this.border.visible
    }

    initLayout() {
        const blob = new Blob([this.imgBlob.blob], { type: `image/${this.imgBlob.type}` })
        const uri = URL.createObjectURL(blob)
        const img = new Image()
        img.src = uri
        img.onload = () => {
            const texture = PIXI.Texture.from(img)
            this.sprite = new PIXI.Sprite(texture)
            this.sprite.x = this.viewRect.width / 2
            this.sprite.y = this.viewRect.height / 2
            this.initScale = Math.min(this.viewRect.width / texture.width, this.viewRect.height / texture.height)
            this.sprite.scale.set(this.initScale, this.initScale)
            this.sprite.anchor.set(0.5, 0.5)
            this.sprite.angle = 0
            this.container.addChild(this.sprite)

            const canvas = texture.baseTexture.resource
            const context = canvas.getContext('2d')
            const imgData = context.getImageData(0, 0, canvas.width, canvas.height)

            const histogram = new Histogram(this.container, imgData)
            histogram.initLayout()
        }
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

const initLayout = async (imgBlobs) => {
    const toolbar = new Toolbar(app)
    toolbar.initLayout(imgBlobs.length > 1)
    blockRects = calculateBlockRects(app.canvas.width, app.canvas.height, imgBlobs.length)
    imgBlobs.forEach((imgBlob, index) => {
        const viewport = new Viewport(app, imgBlob, blockRects[index])
        blockViews.push(viewport)
        viewport.initLayout()
    })
}

watch(() => props.imgBlobs, (imgBlobs) => {
    initLayout(imgBlobs)
})

const initPIXIApp = async () => {
    app = new PIXI.Application()
    await app.init({ background: "#ffffff", resizeTo: window })
    pixiContainer.value.appendChild(app.canvas)

    app.canvas.oncontextmenu = (event) => {
        return false
    }

    app.canvas.addEventListener('wheel', handleWheel, { passive: false })
    app.canvas.addEventListener('dblclick', handleDoubleClick)
    app.canvas.addEventListener('mousedown', handleMouseDown)
    app.canvas.addEventListener('mousemove', handleMouseMove)
    app.canvas.addEventListener('mouseup', handleMouseUp)
}

onMounted(() => {
    initPIXIApp()
})

onUnmounted(() => {
    if (app) {
        app.canvas.removeEventListener('wheel', handleWheel, { passive: false })
        app.canvas.removeEventListener('dblclick', handleDoubleClick)
        app.canvas.removeEventListener('mousedown', handleMouseDown)
        app.canvas.removeEventListener('mousemove', handleMouseMove)
        app.canvas.removeEventListener('mouseup', handleMouseUp)
        app.destroy()
    }
})

</script>

<template>
    <div ref="pixiContainer"></div>
</template>

<style scoped>
.pixi-container {
    width: 100%;
    height: 100%;
}
</style>
