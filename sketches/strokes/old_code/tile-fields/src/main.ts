// @ts-nocheck
import {
	addToLoop,
	startLoop,
} from "../../../../../shared-utils/app/frameLoop";
import { baseEvents } from "../../../../../shared-utils/app/painterState";
import "../../../../../shared-utils/css/fullscreen.css";
import { getBlurByAlphaEffect } from "../../../../../shared-utils/shaders/effects/blur";
import { initCamera } from "../../../../../shared-utils/vr/wasmCamera";
import {
	WasmGeometry,
	wasmGeometryToFormData,
} from "../../../../../shared-utils/wasm/utils";
import init, {
	get_frame_data,
	get_init_data,
	reset_camera,
	setup,
	update_camera,
	update_screen,
} from "../crate/pkg/tvs_sketch_tile_fields";
import { Q } from "./context";
import { PaintingData, setupPainting } from "./render_paintings";
import { canvasShader, groundShader, wallShader } from "./shader";

Q.state.device.sizeMultiplier = window.devicePixelRatio;

interface InitialData {
	paintings: PaintingData[];
	wall_geometry: WasmGeometry;
	ground_geometry: WasmGeometry;
	ceil_mat: number[];
}

const canvasShade = Q.getShade("canvas").update(canvasShader);
const wallShade = Q.getShade("wall").update(wallShader);
const groundShade = Q.getShade("ground").update(groundShader);

init().then(() => {
	setup();

	const data: InitialData = get_init_data(6);

	initCamera(Q, {
		resetCamera: reset_camera,
		updateScreen: update_screen,
		updateTransform: update_camera,
		moveSpeed: 2.5,
	});
	Q.emit(baseEvents.RESIZE);

	const ps = data.paintings;

	const canvasForms = ps.map((d, i) =>
		Q.getForm("canvas" + i).update(wasmGeometryToFormData(d.canvas_geometry))
	);

	const paintingLayers = ps.map((d, i) =>
		setupPainting(i, d.width, d.height, d.tiles)
	);

	const canvasSketches = canvasForms.map((form, i) =>
		Q.getSketch("canvas" + i).update({
			form,
			shade: canvasShade,
			uniforms: {
				modelMat: ps[i].mat,
				painting: () => paintingLayers[i].image(),
			},
		})
	);

	const wallForm = Q.getForm("wall").update(
		wasmGeometryToFormData(data.wall_geometry)
	);

	const wallSketch = Q.getSketch("wall").update({
		form: wallForm,
		shade: wallShade,
		uniforms: ps.map((d) => ({ modelMat: d.mat, color: [1, 1, 1] })),
	});

	const groundForm = Q.getForm("ground").update(
		wasmGeometryToFormData(data.ground_geometry)
	);

	const ceilSketch = Q.getSketch("ceil").update({
		form: groundForm,
		shade: wallShade,
		uniforms: { modelMat: data.ceil_mat, color: [0.97, 0.97, 0.97] },
	});

	const reflectionLayer = Q.getLayer("reflection").update({
		width: Q.gl.drawingBufferWidth / 2,
		height: Q.gl.drawingBufferHeight / 2,
		sketches: [wallSketch, ceilSketch, ...canvasSketches],
		antialias: false,
		drawSettings: {
			enable: [Q.gl.DEPTH_TEST, Q.gl.CULL_FACE],
			clearBits: Q.gl.COLOR_BUFFER_BIT | Q.gl.DEPTH_BUFFER_BIT,
			clearColor: [1, 1, 1, 1],
			cullFace: Q.gl.BACK,
		},
		effects: getBlurByAlphaEffect(Q, "reflectionBlur", {
			strength: 8,
		}),
	});

	const groundSketch = Q.getSketch("ground").update({
		form: groundForm,
		shade: groundShade,
		uniforms: {
			size: () => [Q.gl.drawingBufferWidth, Q.gl.drawingBufferHeight],
			reflection: () => reflectionLayer.image(),
			color: [0.85, 0.82, 0.8],
		},
	});

	const renderLayer = Q.getLayer("render").update({
		sketches: [wallSketch, groundSketch, ceilSketch, ...canvasSketches],
		drawSettings: {
			enable: [Q.gl.DEPTH_TEST, Q.gl.CULL_FACE],
			clearBits: Q.gl.COLOR_BUFFER_BIT | Q.gl.DEPTH_BUFFER_BIT,
			clearColor: [1, 1, 1, 1],
			cullFace: Q.gl.BACK,
		},
	});

	addToLoop(() => {
		// for camera
		Q.emit(baseEvents.FRAME);

		// console.log('wasm get frame data')
		const data = get_frame_data();
		// console.log('wasm get frame data end')

		reflectionLayer.update({
			uniforms: {
				viewProjMat: data.reflected_camera,
			},
		});

		renderLayer.update({
			uniforms: {
				viewProjMat: data.camera,
			},
		});

		// console.log('render room')
		Q.painter.compose(reflectionLayer, renderLayer).show(renderLayer);
		// console.log('render room end')
	}, "render");

	startLoop();
});
