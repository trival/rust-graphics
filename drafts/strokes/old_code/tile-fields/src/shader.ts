import {
	defMain,
	assign,
	vec4,
	mul,
	$y,
	$x,
	Vec4Sym,
	texture,
	sym,
	FloatSym,
	add,
	$w,
	$z,
	pow,
	float,
	vec2,
	sub,
	abs,
	div,
	min,
	clamp,
	length,
	$xy,
	mix,
	$xyz,
	Vec2Sym,
} from '@thi.ng/shader-ast'
import { fit0111, fit1101 } from '@thi.ng/shader-ast-stdlib'
import { defFragment, defShader } from '../../../../../shared-utils/shaders/ast'

export const lineShader = defShader({
	attribs: {
		position: 'vec2',
		localUv: 'vec2',
		uv: 'vec2',
	},
	uniforms: {
		size: 'vec2',
		noiseTex: 'sampler2D',
		color: 'vec3',
	},
	varying: {
		vUv: 'vec2',
		vLocalUv: 'vec2',
	},
	vs: (gl, unis, ins, outs) => [
		defMain(() => [
			assign(outs.vUv, ins.uv),
			assign(outs.vLocalUv, ins.localUv),
			assign(
				gl.gl_Position,
				vec4(mul(fit0111(div(ins.position, unis.size)), vec2(1, -1)), 0, 1),
			),
		]),
	],
	fs: (gl, unis, ins, outs) => {
		let noise: Vec4Sym
		let noiseVal: FloatSym
		return [
			defMain(() => [
				(noise = sym(texture(unis.noiseTex, mul(ins.vUv, vec2(2, 0.8))))),

				(noiseVal = sym($x(noise))),

				// add noise resolutions
				assign(noiseVal, fit0111(noiseVal)),
				assign(noiseVal, add(noiseVal, fit0111($y(noise)))),
				assign(noiseVal, add(noiseVal, fit0111($z(noise)))),
				assign(noiseVal, add(noiseVal, fit0111($w(noise)))),
				assign(noiseVal, fit1101(noiseVal)),

				// adjust intensity
				assign(noiseVal, add(0.15, noiseVal)),
				assign(noiseVal, clamp(noiseVal, float(0), float(1))),
				assign(noiseVal, pow(noiseVal, float(0.25))),

				// fade out edges
				assign(
					noiseVal,
					sub(noiseVal, pow(abs(fit0111($x(ins.vLocalUv))), float(10))),
				),
				assign(
					noiseVal,
					sub(noiseVal, pow(abs(fit0111($y(ins.vUv))), float(10))),
				),

				// fade out at the end
				assign(
					noiseVal,
					mul(
						noiseVal,
						min(float(1), pow(sub(float(1.85), $x(ins.vUv)), float(3))),
					),
				),

				assign(
					outs.fragColor,
					vec4(unis.color, clamp(mul(0.6, noiseVal), float(0), float(1))),
				),
			]),
		]
	},
})

export const bgFrag = defFragment({
	uniforms: {
		color: 'vec3',
	},
	fs(gl, unis, ins, outs) {
		return [defMain(() => [assign(outs.fragColor, vec4(unis.color, 1))])]
	},
})

export const copyFrag = defFragment({
	uniforms: {
		source: 'sampler2D',
	},
	fs(gl, unis, ins, outs) {
		return [
			defMain(() => [assign(outs.fragColor, texture(unis.source, ins.coords))]),
		]
	},
})

export const canvasShader = defShader({
	attribs: {
		position: 'vec3',
		uv: 'vec2',
	},
	uniforms: {
		viewProjMat: 'mat4',
		modelMat: 'mat4',
		painting: 'sampler2D',
	},
	varying: {
		vUv: 'vec2',
	},
	vs: (gl, unis, ins, outs) => [
		defMain(() => [
			assign(outs.vUv, ins.uv),
			assign(
				gl.gl_Position,
				mul(unis.viewProjMat, mul(unis.modelMat, vec4(ins.position, 1))),
			),
		]),
	],
	fs: (gl, unis, ins, outs) => [
		defMain(() => [assign(outs.fragColor, texture(unis.painting, ins.vUv))]),
	],
})

export const wallShader = defShader({
	attribs: {
		position: 'vec3',
		uv: 'vec2',
	},
	uniforms: {
		viewProjMat: 'mat4',
		modelMat: 'mat4',
		color: 'vec3',
	},
	varying: {
		vUv: 'vec2',
	},
	vs: (gl, unis, ins, outs) => [
		defMain(() => [
			assign(outs.vUv, ins.uv),
			assign(
				gl.gl_Position,
				mul(unis.viewProjMat, mul(unis.modelMat, vec4(ins.position, 1))),
			),
		]),
	],
	fs: (gl, unis, ins, outs) => {
		return [
			defMain(() => [
				assign(
					outs.fragColor,
					vec4(
						mul(
							unis.color,
							pow(sub(1, div(length(fit0111(ins.vUv)), 1.5)), float(0.15)),
						),
						1,
					),
				),
			]),
		]
	},
})

export const groundShader = defShader({
	attribs: {
		position: 'vec3',
		uv: 'vec2',
	},
	uniforms: {
		viewProjMat: 'mat4',
		reflection: 'sampler2D',
		size: 'vec2',
		color: 'vec3',
	},
	varying: {
		vUv: 'vec2',
	},
	vs: (gl, unis, ins, outs) => [
		defMain(() => [
			assign(outs.vUv, ins.uv),
			assign(gl.gl_Position, mul(unis.viewProjMat, vec4(ins.position, 1))),
		]),
	],
	fs: (gl, unis, ins, outs) => {
		let reflection: Vec4Sym
		let uv: Vec2Sym
		return [
			defMain(() => [
				(uv = sym(div($xy(gl.gl_FragCoord), unis.size))),
				assign($y(uv), sub(1, $y(uv))),
				(reflection = sym(texture(unis.reflection, uv))),
				assign(
					outs.fragColor,
					vec4(
						mix(
							$xyz(reflection),
							mul(
								unis.color,
								pow(sub(1, div(length(fit0111(ins.vUv)), 1.5)), float(0.25)),
							),
							float(0.9),
						),
						1,
					),
				),
			]),
		]
	},
})
