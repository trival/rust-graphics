// @ts-nocheck
import { randIntInRange } from "tvs-libs/dist/math/random";
import {
	addToLoop,
	startLoop,
} from "../../../../../shared-utils/app/frameLoop";
import "../../../../../shared-utils/css/fullscreen.css";
import init, {
	get_single_painting,
	setup,
} from "../crate/pkg/tvs_sketch_tile_fields";
import { Q } from "./context";
import { PaintingData, setupPainting } from "./render_paintings";

Q.state.device.sizeMultiplier = window.devicePixelRatio;

init().then(() => {
	setup();

	const p: PaintingData = get_single_painting(
		Q.gl.drawingBufferWidth,
		Q.gl.drawingBufferHeight,
		randIntInRange(3, 6)
	);

	const paintingLayer = setupPainting(0, p.width, p.height, p.tiles);

	addToLoop(() => {
		Q.painter.show(paintingLayer);
	}, "render");

	startLoop();
});
