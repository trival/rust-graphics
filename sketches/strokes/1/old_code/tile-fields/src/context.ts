// @ts-nocheck
import { getPainterContext } from "../../../../../shared-utils/app/painterState";

export const canvas = document.getElementById("canvas") as HTMLCanvasElement;
export const Q = getPainterContext(canvas);
