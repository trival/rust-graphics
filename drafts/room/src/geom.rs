use trivalibs::{
	prelude::*,
	rendering::{
		mesh_geometry::{utils::Vert3dUv, MeshBufferType, MeshGeometry},
		shapes::quad::{Quad, QuadVertices},
		BufferedGeometry,
	},
};

pub fn create_ground_plane() -> BufferedGeometry {
	let plane: Quad<Vert3dUv> = Quad::from_dimensions(Vec3::ZERO, Vec2::splat(100.0), Vec3::Y).into();
	let mut geom = MeshGeometry::new();

	geom.add_face4(plane.to_ccw_verts());

	geom.to_buffered_geometry_by_type(MeshBufferType::VertexNormalFaceData)
}
