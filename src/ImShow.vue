<script setup>
import { ref, onMounted, onUnmounted } from "vue"
import * as PIXI from "pixi.js"
import { Button, CheckBox } from '@pixi/ui'
import exifr from 'exifr'

const props = defineProps({
    imgSrcs: {
        type: Array,
        required: true
    }
})

const pixiContainer = ref(null)
let app = null
let blockRects = []
let blockViews = []
let selectedBlockIndex = -1
let isDragging = false
let dragStart = { x: 0, y: 0 }
let animationFrameId = null

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
    }
}

const handleCmpUp = () => {
    if (selectedBlockIndex != -1) {
        blockViews[0].restore()
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
    if (index >= 1 && index < blockViews.length) {
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
    const delta = event.deltaY > 0 ? -1 : 1
    const scaleChange = delta > 0 ? scaleFactor : 1 / scaleFactor

    let wheelBlockIndex = -1
    if (event.ctrlKey) {
        wheelBlockIndex = blockRects.findIndex((blockRect) => isMouseInBlock(event.clientX, event.clientY, blockRect))
    }

    const scaleBlockView = (blockView) => {
        const newScaleX = blockView.sprite.scale.x * scaleChange
        const newScaleY = blockView.sprite.scale.y * scaleChange
        if (newScaleX >= blockView.initScale || newScaleY >= blockView.initScale) {
            const centerX = blockView.viewRect.width / 2
            const centerY = blockView.viewRect.height / 2
            blockView.sprite.x = centerX + (blockView.sprite.x - centerX) * scaleChange
            blockView.sprite.y = centerY + (blockView.sprite.y - centerY) * scaleChange
            blockView.sprite.scale.set(newScaleX, newScaleY)
        }
    }

    if (wheelBlockIndex != -1) {
        scaleBlockView(blockViews[wheelBlockIndex])
    } else {
        blockViews.forEach(scaleBlockView)
    }
}

class Toolbar {
    constructor(app) {
        this.container = new PIXI.Container()
        this.container.x = 0
        this.container.y = 0
        this.container.zIndex = 90
        this.container.visible = false
        app.stage.addChild(this.container)

        this.viewWidth = app.canvas.width
        this.centerX = this.viewWidth / 2
        this.buttons = []
        this.checkboxs = []
    }

    async initLayout(hasCmp = false) {
        PIXI.Assets.add({ alias: 'rotate-left', src: 'src/assets/rotate-left.svg' })
        PIXI.Assets.add({ alias: 'reset', src: 'src/assets/reset.svg' })
        PIXI.Assets.add({ alias: 'left-cmp', src: 'src/assets/left-cmp.svg' })
        PIXI.Assets.add({ alias: 'rotate-right', src: 'src/assets/rotate-right.svg' })
        PIXI.Assets.add({ alias: 'flip-h', src: 'src/assets/flip-horizontal.svg' })
        PIXI.Assets.add({ alias: 'switch-off', src: 'src/assets/switch-off.svg' })
        PIXI.Assets.add({ alias: 'switch-on', src: 'src/assets/switch-on.svg' })

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
        this.addCheckBox('E', cbxTextures['switch-off'], cbxTextures['switch-on'], (checked) => {
            toggleExif()
        })
        this.addCheckBox('H', cbxTextures['switch-off'], cbxTextures['switch-on'], (checked) => {
            toggleHistogram()
        })
        this.addCheckBox('+', cbxTextures['switch-off'], cbxTextures['switch-on'], (checked) => {
            toggleCenterMark()
        })
        this.updatePositions()
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
        onUp && button.onUp.connect(onUp)
    }

    addCheckBox(label, uncheckedTexture, checkedTexture, onChange) {
        const checkbox = new CheckBox({
            text: label,
            style: {
                unchecked: uncheckedTexture,
                checked: checkedTexture,
            }
        })
        checkbox.x = 0
        checkbox.y = 0
        this.container.addChild(checkbox)
        this.checkboxs.push(checkbox)

        onChange && checkbox.onChange.connect(onChange)
    }

    updatePositions(buttonWidth = 25, spacing = 8) {
        const totalButtonWidth = this.buttons.length * (buttonWidth + spacing) - spacing
        const buttonStartX = this.centerX - totalButtonWidth / 2
        this.buttons.forEach((button, index) => {
            button.view.x = buttonStartX + index * (button.width + spacing)
            button.view.y = 0
        })
        this.checkboxs.forEach((checkbox, index) => {
            checkbox.x = this.viewWidth - (index + 1) * (checkbox.width + spacing)
            checkbox.y = 0
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
        this.view.interactive = true
        this.view.on('pointerdown', this.toggleMode, this)

        this.histogramGraphics = new PIXI.Graphics()
        this.view.addChild(this.histogramGraphics)

        this.update(this.texture)
    }

    toggleMode() {
        this.mode = this.mode === 'rgb' ? 'gray' : 'rgb'
        this.update(this.texture)
    }

    update(texture) {
        const canvas = texture.source.resource
        const context = canvas.getContext('2d', { willReadFrequently: true })
        const imgData = context.getImageData(0, 0, canvas.width, canvas.height)

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
    constructor(app, imgSrc, viewRect) {
        this.imgSrc = imgSrc
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

        this.exif = new PIXI.Text('', {fill: 0x00ff00, fontSize: 16 })
        this.exif.x = 5
        this.exif.y = 130
        this.exif.visible = false
        this.exif.zIndex = 100
        this.container.addChild(this.exif)

        this.centerMark = new PIXI.Graphics()
        this.centerMark.beginFill(0xffffff)
        this.centerMark.lineStyle(1, 0xff0000, 1)
        this.centerMark.moveTo(this.viewRect.width / 2, this.viewRect.height / 2 - 10)
        this.centerMark.lineTo(this.viewRect.width / 2, this.viewRect.height / 2 + 10)
        this.centerMark.moveTo(this.viewRect.width / 2 - 10, this.viewRect.height / 2)
        this.centerMark.lineTo(this.viewRect.width / 2 + 10, this.viewRect.height / 2)
        this.centerMark.endFill()
        this.centerMark.visible = false
        this.centerMark.zIndex = 100
        this.container.addChild(this.centerMark)
    }

    flip_h() {
        this.sprite.scale.x = -this.sprite.scale.x
        this.sprite.x = this.viewRect.width - this.sprite.x
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

    toggleExif() {
        this.exif.visible = !this.exif.visible
    }

    toggleHistogram() {
        this.histogram.view.visible = !this.histogram.view.visible
    }

    toggleCenterMark() {
        this.centerMark.visible = !this.centerMark.visible
    }

    update(sprite) {
        this.sprite.visible = false
        this.tmpSprite = new PIXI.Sprite(sprite.texture)
        this.tmpSprite.scale.set(sprite.scale.x, sprite.scale.y)
        this.tmpSprite.angle = sprite.angle
        this.tmpSprite.x = sprite.x
        this.tmpSprite.y = sprite.y
        this.tmpSprite.width = sprite.width
        this.tmpSprite.height = sprite.height
        this.tmpSprite.anchor.set(0.5)
        this.container.addChild(this.tmpSprite)
        this.histogram.update(sprite.texture)
    }

    restore() {
        this.container.removeChild(this.tmpSprite)
        this.sprite.visible = true
        this.histogram.update(this.sprite.texture)
    }

    async readExif(img) {
        try {
            const exifData = await exifr.parse(img)
            if (exifData) {
                let exifInfo = ''
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
                this.exif.text = exifInfo
            } else {
                this.exif.text = 'No EXIF data found'
            }
        } catch (error) {
            console.error('Error reading EXIF:', error)
            this.exif.text = 'Failed to read EXIF'
        }
    }

    initLayout() {
        const img = new Image()
        img.src = this.imgSrc
        img.crossOrigin = "anonymous"
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

            this.histogram = new Histogram(texture)
            this.container.addChild(this.histogram.view)

            this.readExif(img)
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

const initLayout = async (imgSrcs) => {
    const toolbar = new Toolbar(app)
    await toolbar.initLayout(imgSrcs.length > 1)
    blockRects = calculateBlockRects(app.canvas.width, app.canvas.height, imgSrcs.length)
    imgSrcs.forEach((imgSrc, index) => {
        const viewport = new Viewport(app, imgSrc, blockRects[index])
        blockViews.push(viewport)
        viewport.initLayout()
    })
}

const initPIXIApp = async () => {
    app = new PIXI.Application()
    await app.init({ background: "#FFFFFF", resizeTo: window })
    pixiContainer.value.appendChild(app.canvas)

    await initLayout(props.imgSrcs)

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
    if (animationFrameId) {
        cancelAnimationFrame(animationFrameId)
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
