use super::vector::Vector;
use math::round;

// credit to Carl for telling me I could use this. Carl, you're a hero
pub fn render_thing(
    coords: &Vector,
    facing: &Vector,
    size: f32,
    tex_number: u32,
) -> (Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u16>) {
    let indices: Vec<u16> = vec![0, 1, 3, 1, 2, 3];
    let mut vert_coords: Vec<Vector> = Vec::new();

    let og_tl = Vector::new(-1.0 as f32, 1.0 as f32);
    let og_bl = Vector::new(-1.0 as f32, -1.0 as f32);
    let og_br = Vector::new(1.0 as f32, -1.0 as f32);
    let og_tr = Vector::new(1.0 as f32, 1.0 as f32);

    let tl: Vector = og_tl.rotate(&facing).scale_by(size / 2_f32).add(coords);
    let bl: Vector = og_bl.rotate(&facing).scale_by(size / 2_f32).add(coords);
    let br: Vector = og_br.rotate(&facing).scale_by(size / 2_f32).add(coords);
    let tr: Vector = og_tr.rotate(&facing).scale_by(size / 2_f32).add(coords);

    vert_coords.push(tl);
    vert_coords.push(bl);
    vert_coords.push(br);
    vert_coords.push(tr);

    let mut vertices: Vec<[f32; 3]> = Vec::new();

    for i in vert_coords.iter() {
        vertices.push([i.x, i.y, 0_f32]);
    }

    (vertices, get_tex_coords(tex_number), indices)
}

fn get_tex_coords(tex_number: u32) -> Vec<[f32; 2]> {
    // get those tex coords
    // textures oughta be in one png file, of uniform size, arranged thusly:
    /*
    _____________________
    \ 0 \ 1 \ 2 \ 3 \ 4 \
    _____________________
    \ 5 \ 6 \ 7 \ 8 \ 9 \
    _____________________
    etc
     */

    let tex_per_line: f32 = 5_f32;

    // calculated from the above
    let mut tex_coords: Vec<[f32; 2]> = Vec::new();
    let tex_width: f32 = 1_f32 / tex_per_line;

    // fix this, very arbitrary in order to avoid getting a single column of pixels
    // from the next column
    let tl_tex = [
        ((tex_number as f32 % tex_per_line) * tex_width) + 0.002,
        (round::floor(tex_number as f64 / tex_per_line as f64, 0) * 0.2) as f32,
    ];
    let bl_tex = [tl_tex[0], tl_tex[1] + tex_width];
    let br_tex = [tl_tex[0] + tex_width - 0.004, tl_tex[1] + tex_width];
    let tr_tex = [br_tex[0], tl_tex[1]];

    tex_coords.push(tl_tex);
    tex_coords.push(bl_tex);
    tex_coords.push(br_tex);
    tex_coords.push(tr_tex);

    tex_coords
}
