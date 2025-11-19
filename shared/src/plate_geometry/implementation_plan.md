# Plate Geometry Helper

## Overview
The helper constructs a plate-shaped mesh by extruding a user-provided 3D polyline (often on the XZ plane for walls) along an arbitrary direction/length vector while thickening it by a configurable width. Each input point spawns a front and back vertex offset along a normal computed from its neighboring directions and the extrusion vector, so plates remain well-behaved even when the extrusion is tilted away from the ground plane. Stacking multiple translated copies of those loops along the extrusion vector produces the body of the wall. The structure mirrors the existing cuboid helper by exposing methods that return quads for each plate face (front, back, left, right, top, bottom) plus `_f` variants that accept a mapper closure which receives `(position, uvw)` with `(0,0,0)` at the front-top-left corner and `(1,1,1)` at the back-bottom-right corner for precise UV control.

## Implementation Plan
1. **API Surface**
   - Add `PlateGeometry` to `shared/src/plate_geometry/lib.rs` with constructor `new(points: &[Vec3], extrusion: Vec3, width: f32, subdivisions: usize)`.
   - Store precomputed slices (front/back loops at each subdivision level) and expose iterators/helpers returning `Quad3D<P>` similar to `Cuboid`.
2. **Polyline Offsets**
   - For each vertex, compute tangents to previous and next points, average them (handling endpoints by duplicating available neighbor), and cross that averaged tangent with the extrusion vector to get a stable local plate normal; normalize and fall back to a default axis when vectors degenerate.
   - Offset original vertices by `±0.5 * width * normal` to get front/back loops; also keep cumulative arc length for UVs.
3. **Extrusion Slices**
   - Split the extrusion vector into `subdivisions` steps (default 1). Translate both loops by each step to create stacked slices; cache them so face builders can reference arbitrary pairs.
4. **Face Generation**
   - Maintain canonical UVW coordinates for every vertex such that `(0,0,0)` maps to `front_top_left` and `(1,1,1)` maps to `back_bottom_right`, interpolating across curve progress (U), height/extrusion progress (V), and width/front-back offset (W). Store these alongside positions so mapping closures can assign materials consistently.
   - **Front/Back**: Use first and last slices; walk polyline edges and create quads bridging consecutive vertices with consistent winding and normals (`±extrusion_dir`). Provide `_f` variants whose closures receive `(position, uvw)` so callers can derive per-face UV layouts while still referencing the shared canonical cube.
   - **Sides (Left/Right)**: For every polyline edge, stitch corresponding vertices between front/back loops to build width faces; normals align with ±local horizontal axis. Feed the same UVW coordinates into mapper closures to enable flexible texture projection.
   - **Top/Bottom**: For each extrusion slice pair, connect the top and bottom edges derived from front/back loops; normals align with ±extrusion vector and reuse UVW metadata.
5. **Data Exposure**
   - Return borrowed slices or iterators so callers can stream quads without cloning; optionally precompute `Vec<Quad3D<Vec3>>` when `subdivisions` is small.
   - `_f` helpers pass the canonical `uvw` vector for every vertex so downstream code can derive any UV layout (planar projections, trim sheets, etc.) relative to the shared `(0,0,0)`→`(1,1,1)` cube.
   - Document expectations (polyline non-self-intersecting, points ordered, normals derived from neighbor tangents + extrusion direction) and describe how UVs span `[0,1]` along curve/extrusion.
6. **Testing & Validation**
   - Add unit tests verifying normals, quad counts, and UV continuity for simple shapes (rectangle, single-segment wall) plus degenerate handling (duplicate points).
   - Include doc examples showing how to build curved walls rising from ground to target height with specified depth.
