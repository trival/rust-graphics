use serde::Serialize;
use tvs_libs::data_structures::neighbour_list::traits::WithNeighboursTransform;
use tvs_libs::geometry::line_2d::buffered_geometry::{LineBufferedGeometryVec, LineGeometryProps};
use tvs_libs::geometry::line_2d::Line;
use tvs_libs::prelude::*;
use tvs_libs::rendering::buffered_geometry::BufferedGeometry;
use tvs_libs::utils::f32_utils::fit0111;
use tvs_libs::utils::rand_utils::{rand_int, Pick};

#[derive(Clone, Copy, Serialize)]
struct Color {
    hue: f32,
    lightness: f32,
}

#[derive(Clone, Copy)]
struct Tile {
    top: f32,
    left: f32,
    width: f32,
    height: f32,
    color: Color,
}

fn subdivide_tile<F: Fn() -> Color>(
    tile: Tile,
    min_size: f32,
    max_splits: usize,
    split_variance: f32,
    split_direction_variance: f32,
    get_color: F,
) -> Vec<Tile> {
    if tile.width <= min_size || tile.height <= min_size {
        return vec![tile];
    }

    let divide_horizontally =
        tile.width / tile.height + fit0111(random()) * split_direction_variance > 1.;

    let tile_length = if divide_horizontally {
        tile.width
    } else {
        tile.height
    };

    let tile_count = usize::min(max_splits, (tile_length / min_size).floor() as usize);
    if tile_count < 2 {
        return vec![tile];
    }

    let mut tiles = vec![];

    let mut split_ratios = vec![0.];
    split_ratios.append(
        &mut (1..tile_count)
            .map(|i| (fit0111(random()) * 0.5 * split_variance + i as f32) / tile_count as f32)
            .collect(),
    );
    split_ratios.push(1.);

    for ps in split_ratios.windows(2) {
        let l1 = ps[0];
        let l2 = ps[1];
        let start = l1 * tile_length;
        let length = (l2 - l1) * tile_length;
        let new_tile = if divide_horizontally {
            Tile {
                left: start + tile.left,
                width: length,
                top: tile.top,
                height: tile.height,
                color: get_color(),
            }
        } else {
            Tile {
                left: tile.left,
                width: tile.width,
                top: start + tile.top,
                height: length,
                color: get_color(),
            }
        };
        tiles.push(new_tile);
    }

    tiles
}

#[derive(Clone)]
pub struct Painting {
    pub width: usize,
    pub height: usize,
    tiles: Vec<Tile>,
    brush_size: f32,
}

fn random_split(v: Vec<f32>) -> Vec<f32> {
    let idx = rand_int(v.len() - 1);
    let item_i = v[idx];
    let item_ii = v[idx + 1];

    let mut res = vec![];
    for i in 0..=idx {
        res.push(v[i]);
        res.push(item_i.lerp(item_ii, random::<f32>()));
    }
    for i in (idx + 1)..v.len() {
        res.push(v[i]);
    }

    res
}

pub fn create_painting(width: usize, height: usize, color_count: u8) -> Painting {
    let mut hues: Vec<f32> = vec![0., 1.];
    for _ in 0..color_count - 1 {
        hues = random_split(hues);
    }
    hues.pop();

    let hue_shift = random::<f32>();
    let colors = hues
        .into_iter()
        .map(|h| {
            let hue = h + hue_shift;
            Color {
                hue: hue - hue.floor(),
                lightness: (random::<f32>() + random::<f32>()) * 0.4 + 0.1,
            }
        })
        .collect::<Vec<_>>();

    let get_color = || colors.pick().clone();

    let brush_size = height as f32 / 50.0;

    let first_tile = Tile {
        top: 0.,
        left: 0.,
        width: width as f32,
        height: height as f32,
        color: get_color(),
    };

    let mut tiles = vec![first_tile];

    let subdivide_count = rand_int(5) + 1;

    for _ in 0..subdivide_count {
        let mut new_tiles = vec![];
        for tile in tiles {
            let max_splits = rand_int(3) + 2;
            new_tiles.append(&mut subdivide_tile(
                tile,
                brush_size * 3.,
                max_splits,
                0.5,
                0.5,
                get_color,
            ));
        }
        tiles = new_tiles;
    }

    Painting {
        brush_size,
        tiles,
        width,
        height,
    }
}

fn get_line_edges(tile: &Tile, brush_size: f32) -> (Vec<Vec2>, bool) {
    let steps = ((tile.height * 1.3) / brush_size).floor().max(4.);
    let step = tile.height / steps;

    let mut is_left = random::<bool>();

    let delta = || step * 0.2 * fit0111(random());

    let mut points = Vec::new();

    let point_w_offset = step * 0.06;

    points.push(vec2(
        if is_left {
            tile.left - point_w_offset
        } else {
            tile.left + tile.width + point_w_offset
        } + delta() * f32::max(tile.width / (brush_size * 3.), 2.),
        tile.top + step * 1.25 + delta(),
    ));
    is_left = !is_left;

    for i in 1..(steps * 2. - 2.) as usize {
        points.push(vec2(
            if is_left {
                tile.left - point_w_offset
            } else {
                tile.left + tile.width + point_w_offset
            } + delta() * f32::max(tile.width / (brush_size * 3.), 2.),
            tile.top + step * i as f32 * 0.49 + step * 0.75 + delta(),
        ));
        is_left = !is_left;
    }

    points.push(vec2(
        if is_left {
            tile.left - point_w_offset
        } else {
            tile.left + tile.width + point_w_offset
        } + delta() * f32::max(tile.width / (brush_size * 3.), 2.),
        tile.top + step * (steps - 1.) + delta(),
    ));

    (points, is_left)
}

fn make_curve(width: f32, brush_size: f32, p1: Vec2, p2: Vec2, reverse: bool) -> Vec<Vec2> {
    let normal_scale_factor = (brush_size / 6.) * (width / brush_size).min(15.);
    let line = p2 - p1;
    let steps = ((line.length() / 35.).floor() as usize).max(8);
    let normal = if reverse {
        vec2(-line.y, line.x).normalize()
    } else {
        vec2(line.y, -line.x).normalize()
    };
    let p3 = p1 + line * 0.5 + normal * (random::<f32>() - 0.6) * normal_scale_factor;
    let p4 = p1 + line * 0.5 + normal * (random::<f32>() - 0.6) * normal_scale_factor;
    (0..=steps)
        .map(|t| {
            let t = t as f32 / steps as f32;
            // let t = t as f32 / 10.; // This is a really cool bug!!!
            Vec2::cubic_bezier(t, p1, p3, p4, p2)
        })
        .collect()
}

#[derive(Serialize)]
pub struct TileData {
    line_geometries: Vec<Vec<BufferedGeometry>>,
    color: Color,
}

pub fn get_painting_animated_layer(painting: &Painting) -> Vec<TileData> {
    let mut tiles = vec![];

    for tile in painting.tiles.iter() {
        let brush_size = painting
            .brush_size
            .max(tile.height / 10.)
            .min(painting.brush_size * 3.);

        let (points, mut is_left) = get_line_edges(tile, brush_size);

        let mut total_length = 0.0;

        let mut lines = vec![];

        for ps in points.windows(2) {
            let p1 = ps[0];
            let p2 = ps[1];
            let points = make_curve(tile.width, brush_size, p1, p2, is_left);
            is_left = !is_left;

            let mut l = Line::new_offset(brush_size, total_length);
            let mut line_frames = vec![];

            l.add(points[0]);
            for i in 1..points.len() {
                l.add(points[i]);
                line_frames.push(l.clone());
            }

            let line_length = l.line_length();
            total_length += line_length;

            lines.push(line_frames);
        }

        let mut geoms = vec![];

        lines
            .iter()
            .enumerate()
            .with_neighbours()
            .for_each(|(prev, (i, l), next)| {
                let prev_direction = prev.map(|(_, p)| p[p.len() - 1].last().dir);
                let next_direction = next.map(|(_, n)| n[0].first().dir);
                let mut line_geoms = vec![];
                let last = l.len() - 1;
                for (j, frame) in l.iter().enumerate() {
                    line_geoms.push(frame.to_buffered_geometry_with(LineGeometryProps {
                        total_length: Some(total_length),
                        prev_direction,
                        next_direction: if j == last { next_direction } else { None },
                        swap_texture_orientation: i % 2 == 0,
                        ..default()
                    }));
                }
                geoms.push(line_geoms)
            });

        tiles.push(TileData {
            line_geometries: geoms,
            color: tile.color,
        });
    }

    tiles
}

pub fn get_painting_static_layer(painting: &Painting) -> Vec<TileData> {
    let mut tiles = vec![];

    for tile in painting.tiles.iter() {
        let brush_size = painting
            .brush_size
            .max(tile.height / 10.)
            .min(painting.brush_size * 3.);

        let (points, mut is_left) = get_line_edges(tile, brush_size);

        let mut total_length = 0.0;

        let mut lines = vec![];

        for ps in points.windows(2) {
            let p1 = ps[0];
            let p2 = ps[1];
            let ps = make_curve(tile.width, brush_size, p1, p2, is_left);
            is_left = !is_left;

            let mut l = Line::new_offset(brush_size, total_length);

            for i in 0..ps.len() {
                l.add(ps[i]);
            }

            let line_length = l.line_length();
            total_length += line_length;

            lines.push(l);
        }

        tiles.push(TileData {
            line_geometries: vec![lines.to_buffered_geometry_with(LineGeometryProps {
                total_length: Some(total_length),
                ..default()
            })],
            color: tile.color,
        });
    }

    tiles
}
