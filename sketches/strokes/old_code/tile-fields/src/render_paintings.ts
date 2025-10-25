import { adjustHue, hsl, hslToRGB } from 'tvs-libs/dist/graphics/colors'
import { clamp } from 'tvs-libs/dist/math/core'
import { normalRand01, normalRand11 } from 'tvs-libs/dist/math/random'
import { pickRandom, shuffle } from 'tvs-libs/dist/utils/sequence'
import { makeClear } from 'tvs-painter/dist/utils/context'
import { onNextFrame } from '../../../../../shared-utils/app/frameLoop'
import { getNoiseTextureData } from '../../../../../shared-utils/graphics/texture-helpers'
import {
	WasmGeometry,
	wasmGeometryToFormData,
} from '../../../../../shared-utils/wasm/utils'
import {
	get_painting_animation,
	get_painting_static,
} from '../crate/pkg/tvs_sketch_tile_fields'
import { Q } from './context'
import { bgFrag, copyFrag, lineShader } from './shader'

export interface WasmTileData {
	color: {
		hue: number
		lightness: number
	}
	line_geometries: WasmGeometry[][]
}

export interface PaintingData {
	width: number
	height: number
	tiles: WasmTileData[]
	canvas_geometry: WasmGeometry
	mat: number[]
}

const buffer = new Uint8Array(256 * 32)

export const noiseTex = Q.getLayer('noiseTex').update({
	texture: getNoiseTextureData({
		width: 256,
		height: 256,
		startX: 3,
		startY: 3,
		data: {
			magFilter: 'LINEAR',
			minFilter: 'LINEAR',
			wrap: 'REPEAT',
			type: 'UNSIGNED_BYTE',
		},
	}),
})

const copyEffect = Q.getEffect('copy').update({
	frag: copyFrag,
})

const bgInitEffect = Q.getEffect('bg').update({
	frag: bgFrag,
})

const shade = Q.getShade('line').update(lineShader)

function calculateColor(hue: number, lightness: number) {
	return hslToRGB(
		hsl(
			adjustHue(hue + normalRand01() * 0.1),
			Math.pow((Math.random() + Math.random()) / 2, 1.5),
			clamp(0, 1, lightness + normalRand11() * 0.4),
		),
	)
}

export function setupPainting(
	idx: number,
	width: number,
	height: number,
	initialTiles: WasmTileData[],
) {
	const paintingLayer = Q.getLayer('animation' + idx).update({
		width,
		height,
		antialias: false,
		drawSettings: {
			enable: [Q.gl.CULL_FACE, Q.gl.BLEND],
			clearBits: makeClear(Q.gl, 'color'),
			cullFace: Q.gl.BACK,
			clearColor: [1, 1, 1, 1],
			blendEquationSeparate: [Q.gl.FUNC_ADD, Q.gl.MAX],
			blendFuncSeparate: [
				Q.gl.SRC_ALPHA,
				Q.gl.ONE_MINUS_SRC_ALPHA,
				Q.gl.ONE_MINUS_DST_ALPHA,
				Q.gl.ONE_MINUS_SRC_ALPHA,
			],
		},
	})

	const color = pickRandom(initialTiles)!.color

	const backgroundLayer = Q.getLayer('background' + idx).update({
		width,
		height,
		effects: bgInitEffect,
		uniforms: {
			color: calculateColor(color.hue, color.lightness),
		},
	})

	Q.painter.compose(backgroundLayer)

	backgroundLayer.update({
		effects: copyEffect,
		uniforms: {
			source: () => paintingLayer.image(),
		},
	})

	const longestBuffer = initialTiles.reduce(
		(acc, t) => {
			const longest = t.line_geometries[0].reduce(
				(acc, g) => {
					const maxCount = Math.max(acc[0], g.vertex_count)
					const maxSize = Math.max(acc[1], g.vertex_size)
					return [maxCount, maxSize]
				},
				[0, 0],
			)
			if (longest[0] > acc.count) {
				acc.count = longest[0]
				acc.size = Math.max(longest[1], acc.size)
			}
			return acc
		},
		{ count: 0, size: 0 },
	)

	console.log(longestBuffer)

	function renderStaticTile(t: WasmTileData) {
		const color = calculateColor(t.color.hue, t.color.lightness)

		const initialSketches = t.line_geometries[0].map((data, i) => {
			const form = wasmGeometryToFormData(data, 'DYNAMIC')
			form.elements = null
			buffer.set(form.customLayout!.data!.buffer! as Uint8Array)
			form.customLayout!.data!.buffer = buffer

			return Q.getSketch('line' + i).update({
				form: Q.getForm('init_line' + i).update(form),
				shade,
				uniforms: { color },
			})
		})

		paintingLayer.update({
			sketches: [copyEffect, ...initialSketches],
			uniforms: {
				size: [width, height],
				noiseTex: () => noiseTex.image(),
				source: () => backgroundLayer.image(),
			},
		})

		Q.painter.compose(paintingLayer, backgroundLayer)
	}

	for (const t of shuffle(initialTiles)) {
		renderStaticTile(t)
	}

	const secondTiles: WasmTileData[] = get_painting_static(idx)

	for (const t of shuffle(secondTiles)) {
		renderStaticTile(t)
	}

	// console.log('inital painting rendering end', idx)

	function renderLayer() {
		// console.log('wasm get painting animation', idx)
		const data: WasmTileData[] = get_painting_animation(idx)
		// console.log('wasm get painting animation end', idx)

		const tiles = shuffle(data).map(
			({ line_geometries: geometries, color: { hue, lightness } }, i) => {
				const color = calculateColor(hue, lightness)

				return { color, geometries, currentLine: 0, currentFrame: 0, id: i }
			},
		)

		function renderBruches() {
			if (!tiles.length) {
				setTimeout(
					() => onNextFrame(renderLayer, 'rl' + idx),
					Math.random() * 4000 + 100,
				)
				return
			}

			const brush = tiles.pop()!

			function render() {
				const geom = brush.geometries[brush.currentLine]?.[brush.currentFrame]

				if (!geom) {
					setTimeout(
						() => onNextFrame(renderBruches, 'rb' + idx),
						Math.random() * 5000 + 100,
					)
					return
				}

				const form = wasmGeometryToFormData(geom, 'DYNAMIC')
				form.elements = null
				buffer.set(form.customLayout!.data!.buffer! as Uint8Array)
				form.customLayout!.data!.buffer = buffer

				const sketch = Q.getSketch('line').update({
					form: Q.getForm('line' + idx).update(form),
					shade,
					uniforms: {
						color: brush.color,
					},
				})

				paintingLayer.update({
					sketches: [copyEffect, sketch],
					uniforms: {
						size: [width, height],
						noiseTex: () => noiseTex.image(),
						source: () => backgroundLayer.image(),
					},
				})

				const finished =
					brush.geometries[brush.currentLine].length - 1 === brush.currentFrame

				// console.log('rendering painting', idx, finished)
				if (finished) {
					brush.currentFrame = 0
					brush.currentLine++

					Q.painter.compose(paintingLayer, backgroundLayer)
				} else {
					brush.currentFrame++

					Q.painter.compose(paintingLayer)
				}
				// console.log('rendering painting end', idx)

				onNextFrame(render, 'rp' + idx)
			}

			render()
		}

		renderBruches()
	}

	setTimeout(
		() => onNextFrame(renderLayer, 'rl' + idx),
		Math.random() * 5000 + 100,
	)

	return paintingLayer
}
