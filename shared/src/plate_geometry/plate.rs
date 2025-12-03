use trivalibs::prelude::*;
use trivalibs::rendering::shapes::quad::Quad3D;
use trivalibs::data::Position3D;

/// PlateGeometry creates a plate-shaped mesh by extruding a 3D polyline along a direction vector
/// while thickening it by a configurable width.
///
/// The geometry mirrors the Cuboid pattern with canonical UVW coordinates where:
/// - (0,0,0) maps to front-top-left corner
/// - (1,1,1) maps to back-bottom-right corner
/// - U: progress along the polyline curve (0 to 1)
/// - V: progress along the extrusion direction (0 to 1)
/// - W: offset across the width (0 = front, 1 = back)
pub struct PlateGeometry {
	/// Original polyline points
	points: Vec<Vec3>,
	/// Extrusion direction and length
	extrusion: Vec3,
	/// Plate width (thickness), stored for completeness
	#[allow(dead_code)]
	width: f32,
	/// Number of subdivisions along extrusion
	subdivisions: usize,
	
	/// Precomputed front and back loops for each subdivision level
	/// Stored as (front_loop, back_loop) for each subdivision slice
	slices: Vec<(Vec<Vec3>, Vec<Vec3>)>,
	
	/// Arc length for each point along the polyline (for UV mapping)
	arc_lengths: Vec<f32>,
	/// Total arc length of the polyline
	total_arc_length: f32,
}

impl PlateGeometry {
	/// Create a new PlateGeometry from a polyline, extrusion vector, width, and subdivisions.
	///
	/// # Arguments
	/// * `points` - Array of 3D points defining the polyline (at least 2 points required)
	/// * `extrusion` - Vector defining the extrusion direction and length
	/// * `width` - Thickness of the plate
	/// * `subdivisions` - Number of subdivisions along the extrusion (minimum 1)
	pub fn new(points: &[Vec3], extrusion: Vec3, width: f32, subdivisions: usize) -> Self {
		assert!(points.len() >= 2, "PlateGeometry requires at least 2 points");
		assert!(subdivisions >= 1, "PlateGeometry requires at least 1 subdivision");
		assert!(width > 0.0, "PlateGeometry width must be positive");
		
		let points = points.to_vec();
		let subdivisions = subdivisions.max(1);
		
		// Compute arc lengths along the polyline
		let (arc_lengths, total_arc_length) = Self::compute_arc_lengths(&points);
		
		// Compute normals for each point (perpendicular to polyline and extrusion)
		let normals = Self::compute_normals(&points, extrusion);
		
		// Generate front and back loops by offsetting points
		let (front_loop, back_loop) = Self::compute_offset_loops(&points, &normals, width);
		
		// Generate slices along the extrusion
		let slices = Self::compute_slices(&front_loop, &back_loop, extrusion, subdivisions);
		
		Self {
			points,
			extrusion,
			width,
			subdivisions,
			slices,
			arc_lengths,
			total_arc_length,
		}
	}
	
	/// Compute cumulative arc lengths for UV mapping
	fn compute_arc_lengths(points: &[Vec3]) -> (Vec<f32>, f32) {
		let mut arc_lengths = vec![0.0];
		let mut total = 0.0;
		
		for i in 1..points.len() {
			let dist = (points[i] - points[i - 1]).length();
			total += dist;
			arc_lengths.push(total);
		}
		
		(arc_lengths, total)
	}
	
	/// Compute normal vectors for each point in the polyline
	fn compute_normals(points: &[Vec3], extrusion: Vec3) -> Vec<Vec3> {
		let extrusion_norm = extrusion.normalize_or_zero();
		
		// Handle degenerate case
		if extrusion_norm.length_squared() < 1e-6 {
			return vec![Vec3::X; points.len()];
		}
		
		let mut normals = Vec::with_capacity(points.len());
		
		for i in 0..points.len() {
			// Compute tangent based on neighbors
			let tangent = if i == 0 {
				// First point: use direction to next point
				(points[1] - points[0]).normalize_or_zero()
			} else if i == points.len() - 1 {
				// Last point: use direction from previous point
				(points[i] - points[i - 1]).normalize_or_zero()
			} else {
				// Middle points: average of incoming and outgoing directions
				let incoming = (points[i] - points[i - 1]).normalize_or_zero();
				let outgoing = (points[i + 1] - points[i]).normalize_or_zero();
				(incoming + outgoing).normalize_or_zero()
			};
			
			// Normal is perpendicular to both tangent and extrusion
			let normal = tangent.cross(extrusion_norm).normalize_or_zero();
			
			// Handle degenerate case (tangent parallel to extrusion)
			let normal = if normal.length_squared() < 1e-6 {
				// Find a perpendicular vector to extrusion
				let perp = if extrusion_norm.x.abs() < 0.9 {
					Vec3::X
				} else {
					Vec3::Y
				};
				extrusion_norm.cross(perp).normalize_or_zero()
			} else {
				normal
			};
			
			normals.push(normal);
		}
		
		normals
	}
	
	/// Compute front and back loops by offsetting points along their normals
	fn compute_offset_loops(points: &[Vec3], normals: &[Vec3], width: f32) -> (Vec<Vec3>, Vec<Vec3>) {
		let half_width = width * 0.5;
		
		let front_loop: Vec<Vec3> = points.iter()
			.zip(normals.iter())
			.map(|(p, n)| *p + *n * half_width)
			.collect();
		
		let back_loop: Vec<Vec3> = points.iter()
			.zip(normals.iter())
			.map(|(p, n)| *p - *n * half_width)
			.collect();
		
		(front_loop, back_loop)
	}
	
	/// Compute slices along the extrusion direction
	fn compute_slices(
		front_loop: &[Vec3],
		back_loop: &[Vec3],
		extrusion: Vec3,
		subdivisions: usize,
	) -> Vec<(Vec<Vec3>, Vec<Vec3>)> {
		let mut slices = Vec::with_capacity(subdivisions + 1);
		
		for i in 0..=subdivisions {
			let t = i as f32 / subdivisions as f32;
			let offset = extrusion * t;
			
			let front_slice: Vec<Vec3> = front_loop.iter().map(|p| *p + offset).collect();
			let back_slice: Vec<Vec3> = back_loop.iter().map(|p| *p + offset).collect();
			
			slices.push((front_slice, back_slice));
		}
		
		slices
	}
	
	/// Compute UVW coordinates for a vertex
	/// Returns (u, v, w) where:
	/// - u: progress along polyline (0 to 1)
	/// - v: progress along extrusion (0 to 1)
	/// - w: offset across width (0 = front, 1 = back)
	fn compute_uvw(&self, point_idx: usize, slice_idx: usize, is_back: bool) -> Vec3 {
		let u = if self.total_arc_length > 0.0 {
			self.arc_lengths[point_idx] / self.total_arc_length
		} else {
			0.0
		};
		
		let v = slice_idx as f32 / self.subdivisions as f32;
		let w = if is_back { 1.0 } else { 0.0 };
		
		vec3(u, v, w)
	}
	
	/// Get the front face (at the start of extrusion)
	pub fn front_face(&self) -> Vec<Quad3D<Vec3>> {
		self.front_face_f(|pos, _| pos)
	}
	
	/// Get the front face with a mapper function
	/// The mapper receives (position, uvw) where uvw contains canonical coordinates
	pub fn front_face_f<P: Position3D, F: Fn(Vec3, Vec3) -> P>(&self, f: F) -> Vec<Quad3D<P>> {
		let slice_idx = 0;
		let (front_loop, back_loop) = &self.slices[slice_idx];
		let normal = -self.extrusion.normalize_or_zero();
		
		let mut quads = Vec::new();
		
		for i in 0usize..front_loop.len() - 1 {
			let tl = front_loop[i];
			let tr = front_loop[i + 1];
			let bl = back_loop[i];
			let br = back_loop[i + 1];
			
			let uvw_tl = self.compute_uvw(i, slice_idx, false);
			let uvw_tr = self.compute_uvw(i + 1, slice_idx, false);
			let uvw_bl = self.compute_uvw(i, slice_idx, true);
			let uvw_br = self.compute_uvw(i + 1, slice_idx, true);
			
			quads.push(Quad3D {
				top_left: f(tl, uvw_tl),
				top_right: f(tr, uvw_tr),
				bottom_left: f(bl, uvw_bl),
				bottom_right: f(br, uvw_br),
				normal,
			});
		}
		
		quads
	}
	
	/// Get the back face (at the end of extrusion)
	pub fn back_face(&self) -> Vec<Quad3D<Vec3>> {
		self.back_face_f(|pos, _| pos)
	}
	
	/// Get the back face with a mapper function
	pub fn back_face_f<P: Position3D, F: Fn(Vec3, Vec3) -> P>(&self, f: F) -> Vec<Quad3D<P>> {
		let slice_idx = self.subdivisions;
		let (front_loop, back_loop) = &self.slices[slice_idx];
		let normal = self.extrusion.normalize_or_zero();
		
		let mut quads = Vec::new();
		
		for i in 0usize..front_loop.len() - 1 {
			let tl = back_loop[i];
			let tr = back_loop[i + 1];
			let bl = front_loop[i];
			let br = front_loop[i + 1];
			
			let uvw_tl = self.compute_uvw(i, slice_idx, true);
			let uvw_tr = self.compute_uvw(i + 1, slice_idx, true);
			let uvw_bl = self.compute_uvw(i, slice_idx, false);
			let uvw_br = self.compute_uvw(i + 1, slice_idx, false);
			
			quads.push(Quad3D {
				top_left: f(tl, uvw_tl),
				top_right: f(tr, uvw_tr),
				bottom_left: f(bl, uvw_bl),
				bottom_right: f(br, uvw_br),
				normal,
			});
		}
		
		quads
	}
	
	/// Get the left side face (front side of the plate)
	pub fn left_face(&self) -> Vec<Quad3D<Vec3>> {
		self.left_face_f(|pos, _| pos)
	}
	
	/// Get the left side face with a mapper function
	pub fn left_face_f<P: Position3D, F: Fn(Vec3, Vec3) -> P>(&self, f: F) -> Vec<Quad3D<P>> {
		let mut quads = Vec::new();
		
		for slice_idx in 0..self.subdivisions {
			let (front_loop_curr, _) = &self.slices[slice_idx];
			let (front_loop_next, _) = &self.slices[slice_idx + 1];
			
			for i in 0usize..front_loop_curr.len() - 1 {
				// Compute normal for this edge segment
				let edge_dir = (front_loop_curr[i + 1] - front_loop_curr[i]).normalize_or_zero();
				let normal = edge_dir.cross(self.extrusion.normalize_or_zero()).normalize_or_zero();
				
				let tl = front_loop_curr[i];
				let tr = front_loop_curr[i + 1];
				let bl = front_loop_next[i];
				let br = front_loop_next[i + 1];
				
				let uvw_tl = self.compute_uvw(i, slice_idx, false);
				let uvw_tr = self.compute_uvw(i + 1, slice_idx, false);
				let uvw_bl = self.compute_uvw(i, slice_idx + 1, false);
				let uvw_br = self.compute_uvw(i + 1, slice_idx + 1, false);
				
				quads.push(Quad3D {
					top_left: f(tl, uvw_tl),
					top_right: f(tr, uvw_tr),
					bottom_left: f(bl, uvw_bl),
					bottom_right: f(br, uvw_br),
					normal,
				});
			}
		}
		
		quads
	}
	
	/// Get the right side face (back side of the plate)
	pub fn right_face(&self) -> Vec<Quad3D<Vec3>> {
		self.right_face_f(|pos, _| pos)
	}
	
	/// Get the right side face with a mapper function
	pub fn right_face_f<P: Position3D, F: Fn(Vec3, Vec3) -> P>(&self, f: F) -> Vec<Quad3D<P>> {
		let mut quads = Vec::new();
		
		for slice_idx in 0..self.subdivisions {
			let (_, back_loop_curr) = &self.slices[slice_idx];
			let (_, back_loop_next) = &self.slices[slice_idx + 1];
			
			for i in 0usize..back_loop_curr.len() - 1 {
				// Compute normal for this edge segment (opposite to left face)
				let edge_dir = (back_loop_curr[i + 1] - back_loop_curr[i]).normalize_or_zero();
				let normal = -edge_dir.cross(self.extrusion.normalize_or_zero()).normalize_or_zero();
				
				let tl = back_loop_curr[i + 1];
				let tr = back_loop_curr[i];
				let bl = back_loop_next[i + 1];
				let br = back_loop_next[i];
				
				let uvw_tl = self.compute_uvw(i + 1, slice_idx, true);
				let uvw_tr = self.compute_uvw(i, slice_idx, true);
				let uvw_bl = self.compute_uvw(i + 1, slice_idx + 1, true);
				let uvw_br = self.compute_uvw(i, slice_idx + 1, true);
				
				quads.push(Quad3D {
					top_left: f(tl, uvw_tl),
					top_right: f(tr, uvw_tr),
					bottom_left: f(bl, uvw_bl),
					bottom_right: f(br, uvw_br),
					normal,
				});
			}
		}
		
		quads
	}
	
	/// Get the top edge faces (connecting the ends of the polyline)
	pub fn top_face(&self) -> Vec<Quad3D<Vec3>> {
		self.top_face_f(|pos, _| pos)
	}
	
	/// Get the top edge face with a mapper function (start of polyline)
	pub fn top_face_f<P: Position3D, F: Fn(Vec3, Vec3) -> P>(&self, f: F) -> Vec<Quad3D<P>> {
		let mut quads = Vec::new();
		let point_idx = 0;
		
		// Compute normal (perpendicular to extrusion and edge)
		let edge_dir = (self.slices[0].1[point_idx] - self.slices[0].0[point_idx]).normalize_or_zero();
		let normal = -self.extrusion.normalize_or_zero().cross(edge_dir).normalize_or_zero();
		
		for slice_idx in 0..self.subdivisions {
			let (front_curr, back_curr) = &self.slices[slice_idx];
			let (front_next, back_next) = &self.slices[slice_idx + 1];
			
			let tl = front_curr[point_idx];
			let tr = back_curr[point_idx];
			let bl = front_next[point_idx];
			let br = back_next[point_idx];
			
			let uvw_tl = self.compute_uvw(point_idx, slice_idx, false);
			let uvw_tr = self.compute_uvw(point_idx, slice_idx, true);
			let uvw_bl = self.compute_uvw(point_idx, slice_idx + 1, false);
			let uvw_br = self.compute_uvw(point_idx, slice_idx + 1, true);
			
			quads.push(Quad3D {
				top_left: f(tl, uvw_tl),
				top_right: f(tr, uvw_tr),
				bottom_left: f(bl, uvw_bl),
				bottom_right: f(br, uvw_br),
				normal,
			});
		}
		
		quads
	}
	
	/// Get the bottom edge faces (connecting the ends of the polyline)
	pub fn bottom_face(&self) -> Vec<Quad3D<Vec3>> {
		self.bottom_face_f(|pos, _| pos)
	}
	
	/// Get the bottom edge face with a mapper function (end of polyline)
	pub fn bottom_face_f<P: Position3D, F: Fn(Vec3, Vec3) -> P>(&self, f: F) -> Vec<Quad3D<P>> {
		let mut quads = Vec::new();
		let point_idx = self.points.len() - 1;
		
		// Compute normal (perpendicular to extrusion and edge)
		let edge_dir = (self.slices[0].1[point_idx] - self.slices[0].0[point_idx]).normalize_or_zero();
		let normal = self.extrusion.normalize_or_zero().cross(edge_dir).normalize_or_zero();
		
		for slice_idx in 0..self.subdivisions {
			let (front_curr, back_curr) = &self.slices[slice_idx];
			let (front_next, back_next) = &self.slices[slice_idx + 1];
			
			let tl = back_curr[point_idx];
			let tr = front_curr[point_idx];
			let bl = back_next[point_idx];
			let br = front_next[point_idx];
			
			let uvw_tl = self.compute_uvw(point_idx, slice_idx, true);
			let uvw_tr = self.compute_uvw(point_idx, slice_idx, false);
			let uvw_bl = self.compute_uvw(point_idx, slice_idx + 1, true);
			let uvw_br = self.compute_uvw(point_idx, slice_idx + 1, false);
			
			quads.push(Quad3D {
				top_left: f(tl, uvw_tl),
				top_right: f(tr, uvw_tr),
				bottom_left: f(bl, uvw_bl),
				bottom_right: f(br, uvw_br),
				normal,
			});
		}
		
		quads
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	
	// Helper struct for testing UVW coordinates
	#[derive(Debug, Clone, Copy)]
	struct VertUvw {
		pos: Vec3,
		uvw: Vec3,
	}
	
	impl Position3D for VertUvw {
		fn position(&self) -> Vec3 {
			self.pos
		}
	}
	
	#[test]
	fn test_plate_geometry_simple() {
		let points = vec![
			vec3(0.0, 0.0, 0.0),
			vec3(1.0, 0.0, 0.0),
		];
		let extrusion = vec3(0.0, 1.0, 0.0);
		let width = 0.5;
		let subdivisions = 1;
		
		let plate = PlateGeometry::new(&points, extrusion, width, subdivisions);
		
		// Verify basic properties
		assert_eq!(plate.points.len(), 2);
		assert_eq!(plate.subdivisions, 1);
		assert_eq!(plate.slices.len(), 2); // subdivisions + 1
		
		// Verify front face
		let front_quads = plate.front_face();
		assert_eq!(front_quads.len(), 1); // points.len() - 1
		
		// Verify back face
		let back_quads = plate.back_face();
		assert_eq!(back_quads.len(), 1);
		
		// Verify side faces
		let left_quads = plate.left_face();
		assert_eq!(left_quads.len(), 1); // (points.len() - 1) * subdivisions
		
		let right_quads = plate.right_face();
		assert_eq!(right_quads.len(), 1);
		
		// Verify top/bottom faces
		let top_quads = plate.top_face();
		assert_eq!(top_quads.len(), 1); // subdivisions
		
		let bottom_quads = plate.bottom_face();
		assert_eq!(bottom_quads.len(), 1);
	}
	
	#[test]
	fn test_plate_geometry_curved() {
		let points = vec![
			vec3(0.0, 0.0, 0.0),
			vec3(1.0, 0.0, 0.0),
			vec3(2.0, 0.0, 1.0),
		];
		let extrusion = vec3(0.0, 2.0, 0.0);
		let width = 0.4;
		let subdivisions = 2;
		
		let plate = PlateGeometry::new(&points, extrusion, width, subdivisions);
		
		// Verify basic properties
		assert_eq!(plate.points.len(), 3);
		assert_eq!(plate.subdivisions, 2);
		assert_eq!(plate.slices.len(), 3); // subdivisions + 1
		
		// Verify front face has correct number of quads
		let front_quads = plate.front_face();
		assert_eq!(front_quads.len(), 2); // points.len() - 1
		
		// Verify side faces
		let left_quads = plate.left_face();
		assert_eq!(left_quads.len(), 4); // (points.len() - 1) * subdivisions
		
		let right_quads = plate.right_face();
		assert_eq!(right_quads.len(), 4);
	}
	
	#[test]
	fn test_plate_geometry_normals() {
		let points = vec![
			vec3(0.0, 0.0, 0.0),
			vec3(1.0, 0.0, 0.0),
		];
		let extrusion = vec3(0.0, 1.0, 0.0);
		let width = 0.5;
		let subdivisions = 1;
		
		let plate = PlateGeometry::new(&points, extrusion, width, subdivisions);
		
		// Verify front face normal (opposite of extrusion)
		let front_quads = plate.front_face();
		assert_eq!(front_quads[0].normal, vec3(0.0, -1.0, 0.0));
		
		// Verify back face normal (same as extrusion)
		let back_quads = plate.back_face();
		assert_eq!(back_quads[0].normal, vec3(0.0, 1.0, 0.0));
	}
	
	#[test]
	fn test_plate_geometry_uvw_coordinates() {
		let points = vec![
			vec3(0.0, 0.0, 0.0),
			vec3(1.0, 0.0, 0.0),
		];
		let extrusion = vec3(0.0, 1.0, 0.0);
		let width = 0.5;
		let subdivisions = 1;
		
		let plate = PlateGeometry::new(&points, extrusion, width, subdivisions);
		
		// Test UVW mapping through _f variant
		let front_quads = plate.front_face_f(|pos, uvw| VertUvw { pos, uvw });
		
		// First quad should have uvw coordinates
		let quad = &front_quads[0];
		
		// Top left: start of polyline, start of extrusion, front side
		let uvw_tl = quad.top_left.uvw;
		assert_eq!(uvw_tl.x, 0.0); // u = 0 (start of polyline)
		assert_eq!(uvw_tl.y, 0.0); // v = 0 (start of extrusion)
		assert_eq!(uvw_tl.z, 0.0); // w = 0 (front side)
		
		// Top right: end of polyline, start of extrusion, front side
		let uvw_tr = quad.top_right.uvw;
		assert_eq!(uvw_tr.x, 1.0); // u = 1 (end of polyline)
		assert_eq!(uvw_tr.y, 0.0); // v = 0 (start of extrusion)
		assert_eq!(uvw_tr.z, 0.0); // w = 0 (front side)
		
		// Bottom left: start of polyline, start of extrusion, back side
		let uvw_bl = quad.bottom_left.uvw;
		assert_eq!(uvw_bl.x, 0.0); // u = 0 (start of polyline)
		assert_eq!(uvw_bl.y, 0.0); // v = 0 (start of extrusion)
		assert_eq!(uvw_bl.z, 1.0); // w = 1 (back side)
	}
	
	#[test]
	#[should_panic(expected = "PlateGeometry requires at least 2 points")]
	fn test_plate_geometry_too_few_points() {
		let points = vec![vec3(0.0, 0.0, 0.0)];
		let extrusion = vec3(0.0, 1.0, 0.0);
		let width = 0.5;
		let subdivisions = 1;
		
		PlateGeometry::new(&points, extrusion, width, subdivisions);
	}
	
	#[test]
	#[should_panic(expected = "PlateGeometry width must be positive")]
	fn test_plate_geometry_zero_width() {
		let points = vec![
			vec3(0.0, 0.0, 0.0),
			vec3(1.0, 0.0, 0.0),
		];
		let extrusion = vec3(0.0, 1.0, 0.0);
		let width = 0.0;
		let subdivisions = 1;
		
		PlateGeometry::new(&points, extrusion, width, subdivisions);
	}
}
