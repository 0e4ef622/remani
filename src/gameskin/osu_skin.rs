//! Osu skin directory parser module

// TODO add support for hd textures (scaling shouldn't be an issue, just finding them and loading
// them which shouldn't be hard but I'm just lazy

use graphics::{
    draw_state::{self, DrawState},
    image::Image,
    math, Graphics,
};
use texture::{CreateTexture, Format, ImageSize, TextureSettings, TextureOp};

use std::{
    collections::HashMap, error, fmt, fs::File, io::BufRead, io::BufReader, path, rc::Rc, str, time,
};

use crate::judgement::Judgement;
use super::{ParseError, GameSkin};

#[derive(Copy, Clone, Debug)]
enum NoteBodyStyle {
    Stretch,
    CascadeFromTop,
    CascadeFromBottom,
}

impl str::FromStr for NoteBodyStyle {
    type Err = NoteBodyStyleParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(NoteBodyStyle::Stretch),
            "1" => Ok(NoteBodyStyle::CascadeFromTop),
            "2" => Ok(NoteBodyStyle::CascadeFromBottom),
            _ => Err(NoteBodyStyleParseError),
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct NoteBodyStyleParseError;

impl fmt::Display for NoteBodyStyleParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid NoteBodyStyle variant")
    }
}

#[derive(Copy, Clone, Debug)]
enum HitAnimState {
    SingleNote(time::Instant),
    LongNote(time::Instant),
    /// Time when the animation started, and time when the animation was told to
    /// finish whatever is left.
    LongNoteFinal(time::Instant, time::Instant),
    None,
}

/// Holds skin data, such as note images and what not.
struct OsuSkinTextures<T> {
    miss: Rc<[Rc<T>]>,
    hit50: Rc<[Rc<T>]>,
    hit100: Rc<[Rc<T>]>,
    hit200: Rc<[Rc<T>]>,
    hit300: Rc<[Rc<T>]>,
    hit300g: Rc<[Rc<T>]>,

    /// The animation played when a single note is pressed
    lighting_n: Rc<[Rc<T>]>,

    /// The animation played when a long note is pressed
    lighting_l: Rc<[Rc<T>]>,

    /// The images virtual keys under the judgement line.
    keys: [Rc<T>; 7],

    /// The images of the virtual keys under the judgement line when the
    /// corresponding key on the keyboard is pressed.
    keys_d: [Rc<T>; 7],

    /// The notes' images.
    notes: [Rc<[Rc<T>]>; 7],

    /// The long notes' ends' images.
    long_notes_head: [Rc<[Rc<T>]>; 7],

    /// The long notes' bodies' images.
    long_notes_body: [Rc<[Rc<T>]>; 7],

    /// The long notes' tails' images.
    long_notes_tail: [Option<Rc<[Rc<T>]>>; 7],

    /// The stage light animation images
    stage_light: Rc<[Rc<T>]>,

    /// The stage components.
    stage_hint: Rc<[Rc<T>]>,
    stage_left: Rc<T>,
    stage_right: Rc<T>,
    stage_bottom: Option<Rc<[Rc<T>]>>,
}

/// Various information related to how to draw components. All the numbers are
/// taken unmodified from the skin.ini file. Scaling happens in the drawing
/// functions.
struct OsuSkinConfig {
    column_start: u16,
    column_width: [u16; 7],
    column_spacing: [u16; 6],
    column_line_width: [u16; 8],
    colour_column_line: [u8; 4],
    hit_position: u16,
    score_position: u16,
    light_position: u16,
    width_for_note_height_scale: f64,
    note_body_style: [NoteBodyStyle; 7],

    colour_light: [[u8; 3]; 7],
    // TODO
    // lighting_n_width: [u16; 7],
    // lighting_l_width: [u16; 7],
    // combo_position: u16,
    // judgement_line: bool,

    // low priority
    // special_style: SpecialStyle,
    // keys_under_notes: bool,
}

struct OsuAnimStates {
    keys_last_down_time: [Option<time::Instant>; 7],
    hit_anim: [HitAnimState; 7],
}

struct OsuSkin<G: Graphics> {
    textures: OsuSkinTextures<G::Texture>,
    config: OsuSkinConfig,
    anim_states: OsuAnimStates,

    /// judgement, time of first frame
    judgement: Option<(Judgement, time::Instant)>,
}

impl<G: Graphics> GameSkin<G> for OsuSkin<G> {
    fn draw_play_scene(
        &mut self,
        transform: math::Matrix2d,
        g: &mut G,
        stage_height: f64,
        keys_down: &[bool; 7],
        // column index, start pos, end pos
        notes: &[(usize, f64, Option<f64>)],
    ) {
        let draw_state = &DrawState::default();

        self.draw_track(draw_state, transform, g, stage_height);
        self.draw_keys(draw_state, transform, g, stage_height, keys_down);
        for &(column, pos, end_pos) in notes {
            if let Some(end_p) = end_pos {
                self.draw_long_note(draw_state, transform, g, stage_height, pos, end_p, column);
            } else {
                self.draw_note(draw_state, transform, g, stage_height, pos, column);
            }
        }

        self.draw_hit_anims(
            &DrawState::default().blend(draw_state::Blend::Add),
            transform,
            g,
            stage_height,
        );

        // Draw judgement
        if let Some((judgement, time)) = self.judgement {
            let elapsed = time.elapsed();

            if elapsed <= time::Duration::from_millis(200) {
                // the "burst" animation
                let scale = if elapsed <= time::Duration::from_millis(50) {
                    1.5 - elapsed.subsec_nanos() as f64 / 50_000_000.0 / 2.0
                } else if elapsed <= time::Duration::from_millis(160) {
                    1.0
                } else {
                    1.0 - (elapsed.subsec_nanos() - 160_000_000) as f64 / 150_000_000.0
                };
                match judgement {
                    Judgement::Miss => self.draw_miss(draw_state, transform, g, stage_height),
                    Judgement::Bad => remani_warn!("draw Bad judgement is not implemented for osu skin"), // TODO
                    Judgement::Good => remani_warn!("draw Bad judgement is not implemented for osu skin"),
                    Judgement::Perfect => {
                        self.draw_perfect(draw_state, transform, scale, g, stage_height, elapsed)
                    }
                };
            } else {
                self.judgement = None;
            }
        }
    }

    fn draw_judgement(&mut self, _column: usize, judgement: Judgement) {
        self.judgement = Some((judgement, time::Instant::now()));
    }

    fn key_down(&mut self, column: usize) {
        self.anim_states.keys_last_down_time[column] = None;
    }

    fn key_up(&mut self, column: usize) {
        self.anim_states.keys_last_down_time[column] = Some(time::Instant::now());
    }

    fn single_note_hit_anim(&mut self, column: usize) {
        self.anim_states.hit_anim[column] = HitAnimState::SingleNote(time::Instant::now());
    }

    fn long_note_hit_anim_start(&mut self, column: usize) {
        self.anim_states.hit_anim[column] = HitAnimState::LongNote(time::Instant::now());
    }

    fn long_note_hit_anim_stop(&mut self, column: usize) {
        match self.anim_states.hit_anim[column] {
            HitAnimState::LongNote(time) => self.anim_states.hit_anim[column] = HitAnimState::LongNoteFinal(time, time::Instant::now()),
            _ => (),
        }
    }
}

impl<G: Graphics> OsuSkin<G> {
    fn draw_note(
        &self,
        draw_state: &DrawState,
        transform: math::Matrix2d,
        g: &mut G,
        stage_h: f64,
        pos: f64,
        column_index: usize,
    ) {
        // TODO mania-note is animatable

        let scale = stage_h / 480.0;
        let hit_p = self.config.hit_position as f64 * scale;

        let note_w = self.config.column_width[column_index] as f64 * scale;
        let note_h = self.config.width_for_note_height_scale * scale;
        // Calculate X position from column start, column width sum, and column spacing sum up to
        // column_index
        let note_x = scale
            * (self.config.column_start as f64
                + self.config.column_width[0..column_index]
                    .iter()
                    .sum::<u16>() as f64
                + self.config.column_spacing[0..column_index]
                    .iter()
                    .sum::<u16>() as f64);

        let note_y = hit_p * (1.0 - pos) - note_h;

        let note = &*self.textures.notes[column_index][0];
        let note_img = Image::new().rect([note_x, note_y, note_w, note_h]);
        note_img.draw(note, draw_state, transform, g);
    }
    fn draw_long_note(
        &self,
        draw_state: &DrawState,
        transform: math::Matrix2d,
        g: &mut G,
        stage_h: f64,
        pos: f64,
        end_pos: f64,
        column_index: usize,
    ) {
        // TODO mania-note#L is animatable

        let scale = stage_h / 480.0;
        let scale2 = stage_h / 768.0; // long note body height when cascading is scaled with this
        let hit_p = self.config.hit_position as f64 * scale;

        let note_w = self.config.column_width[column_index] as f64 * scale;
        // Calculate X position from column start, column width sum, and column spacing sum up to
        // column_index
        let note_x = scale
            * (self.config.column_start as f64
                + self.config.column_width[0..column_index]
                    .iter()
                    .sum::<u16>() as f64
                + self.config.column_spacing[0..column_index]
                    .iter()
                    .sum::<u16>() as f64);
        // Theoretical long note bottom
        let real_bottom_y = hit_p * (1.0 - pos);
        // Long note bottom but clamped at the hit position/judgement line
        // TODO Don't clamp if the long note isn't being held
        let bottom_y = if pos < 0.0 { hit_p } else { real_bottom_y };
        let top_y = hit_p * (1.0 - end_pos);

        let note_head = &*self.textures.long_notes_head[column_index][0];
        let note_tail = self.textures.long_notes_tail[column_index].as_ref().map(|v| &*v[0]);
        let note_body = &*self.textures.long_notes_body[column_index][0];

        let note_body_h = note_body.get_height() as f64 * scale2;
        let note_end_h = self.config.width_for_note_height_scale * scale;
        let note_head_y = bottom_y - note_end_h;
        let note_tail_y = top_y - note_end_h;

        let note_head_img = Image::new().rect([note_x, note_head_y, note_w, note_end_h]);
        let note_tail_img = Image::new().rect([note_x, note_tail_y, note_w, note_end_h]);

        match self.config.note_body_style[column_index] {
            // Note body image is stretched to the height of the note
            NoteBodyStyle::Stretch => {
                let note_body_img = Image::new()
                    .src_rect([
                        0.0,
                        0.0,
                        note_body.get_width() as f64,
                        ((bottom_y - top_y) / (real_bottom_y - top_y)
                            * (note_body.get_height() as f64)),
                    ]).rect([
                        note_x,
                        top_y - note_end_h / 2.0,
                        note_w,
                        bottom_y - top_y,
                    ]);
                note_body_img.draw(note_body, draw_state, transform, g);
            }
            // Note body image is repeated, starting from the top
            NoteBodyStyle::CascadeFromTop => {
                let mut rect = [note_x, top_y - note_end_h / 2.0, note_w, note_body_h];
                let mut i = 0.0;
                let mut note_body_img = Image::new();
                while i < bottom_y - top_y - note_body_h {
                    note_body_img = note_body_img.rect(rect);
                    note_body_img.draw(note_body, draw_state, transform, g);
                    rect[1] += note_body_h;
                    i += note_body_h;
                }
                let mut mod_rect = rect;
                mod_rect[3] = bottom_y - top_y - i;
                let src_rect = [0.0, 0.0, note_body.get_width() as f64, mod_rect[3]];
                note_body_img = note_body_img.src_rect(src_rect).rect(mod_rect);
                note_body_img.draw(note_body, draw_state, transform, g);
            }
            // Note body image is repeated, starting from the bottom
            NoteBodyStyle::CascadeFromBottom => {
                let mut rect = [note_x, top_y - note_end_h / 2.0, note_w, note_body_h];
                let mut note_body_img = Image::new();

                let offset = (real_bottom_y - top_y) % note_body_h;

                let mut mod_rect = rect;
                mod_rect[3] = offset;
                let src_rect = [
                    0.0,
                    offset / scale2,
                    note_body.get_width() as f64,
                    -(mod_rect[3] / scale2),
                ];
                note_body_img = note_body_img.src_rect(src_rect).rect(mod_rect);
                note_body_img.draw(note_body, draw_state, transform, g);

                note_body_img = Image::new();

                rect[1] += offset;
                let mut i = offset;

                // The rectangle is upside down to simplify the thinking, and the image source
                // rectangle is also upside down so it's fine
                let upside_down_rect = [
                    0.0,
                    note_body.get_height() as f64,
                    note_body.get_width() as f64,
                    -(note_body.get_height() as f64),
                ];

                while i < bottom_y - top_y - note_body_h {
                    note_body_img = note_body_img.src_rect(upside_down_rect).rect(rect);
                    note_body_img.draw(note_body, draw_state, transform, g);
                    rect[1] += note_body_h;
                    i += note_body_h;
                }

                mod_rect = rect;
                mod_rect[3] = bottom_y - top_y - i;
                let src_rect = [
                    0.0,
                    note_body.get_height() as f64,
                    note_body.get_width() as f64,
                    -mod_rect[3],
                ];
                note_body_img = note_body_img.src_rect(src_rect).rect(mod_rect);
                note_body_img.draw(note_body, draw_state, transform, g);

                let note_body_img =
                    Image::new().rect([note_x, top_y - note_end_h / 2.0, note_w, bottom_y - top_y]);
                note_body_img.draw(note_body, draw_state, transform, g);
            }
        }

        note_head_img.draw(note_head, draw_state, transform, g);

        // If there's a separate note tail image, use it, otherwise, flip the note head image and
        // use it
        if let Some(note_tail) = note_tail {
            note_tail_img.draw(note_tail, draw_state, transform, g);
        } else {
            note_tail_img
                .src_rect([
                    0.0,
                    note_head.get_height() as f64,
                    note_head.get_width() as f64,
                    -(note_head.get_height() as f64),
                ]).draw(note_head, draw_state, transform, g);
        }
    }

    fn draw_track(
        &self,
        draw_state: &DrawState,
        transform: math::Matrix2d,
        g: &mut G,
        stage_h: f64,
    ) {
        // TODO mania-stage-bottom, mania-stage-light, and mania-stage-hint are all animatable

        let scale = stage_h / 480.0;

        // Apparently some things are based on a height of 480, and other things are based on a
        // height of 768. .-.
        let scale2 = stage_h / 768.0;

        let column_width_sum = (self.config.column_width.iter().sum::<u16>() as f64
            + self.config.column_spacing.iter().sum::<u16>() as f64)
            * scale;
        let column_start = self.config.column_start as f64 * scale;
        let stage_hint_height = self.textures.stage_hint[0].get_height() as f64 * scale;
        let stage_l_width = self.textures.stage_left.get_width() as f64 * scale2;
        let stage_r_width = self.textures.stage_right.get_width() as f64 * scale2;

        let stage_l_img =
            Image::new().rect([column_start - stage_l_width, 0.0, stage_l_width, stage_h]);
        let stage_r_img =
            Image::new().rect([column_start + column_width_sum, 0.0, stage_r_width, stage_h]);
        let stage_hint_img = Image::new().rect([
            column_start,
            self.config.hit_position as f64 * scale - stage_hint_height / 2.0,
            column_width_sum,
            stage_hint_height,
        ]);

        stage_hint_img.draw(
            &*self.textures.stage_hint[0],
            draw_state,
            transform,
            g,
        );
        stage_l_img.draw(&*self.textures.stage_left, draw_state, transform, g);
        stage_r_img.draw(&*self.textures.stage_right, draw_state, transform, g);

        if let Some(ref v) = self.textures.stage_bottom {
            let stage_bottom = &*v[0];
            let stage_b_width = stage_bottom.get_width() as f64 * scale;
            let stage_b_height = stage_bottom.get_height() as f64 * scale;
            let stage_b_img = Image::new().rect([
                column_start + column_width_sum / 2.0 - stage_b_width / 2.0,
                stage_h - stage_b_height,
                stage_b_width,
                stage_b_height,
            ]);
            stage_b_img.draw(stage_bottom, draw_state, transform, g);
        }
    }

    fn draw_keys(
        &self,
        draw_state: &DrawState,
        transform: math::Matrix2d,
        g: &mut G,
        stage_h: f64,
        pressed: &[bool; 7],
    ) {
        let scale = stage_h / 480.0;
        let scale2 = stage_h / 768.0;

        let hit_position = self.config.hit_position as f64 * scale;

        for (i, key_pressed) in pressed.iter().enumerate() {
            let key_texture = if *key_pressed {
                self.textures.keys_d[i].as_ref()
            } else {
                self.textures.keys[i].as_ref()
            };
            let key_width = self.config.column_width[i] as f64 * scale;
            let key_height = key_texture.get_height() as f64 * scale2;
            let key_x = scale
                * (self.config.column_start as f64
                    + self.config.column_width[0..i].iter().sum::<u16>() as f64
                    + self.config.column_spacing[0..i].iter().sum::<u16>() as f64);
            let key_y = stage_h - key_height;
            let key_img = Image::new().rect([key_x, key_y, key_width, key_height]);

            let mut color = [
                self.config.colour_light[i][0] as f32 / 255.0,
                self.config.colour_light[i][1] as f32 / 255.0,
                self.config.colour_light[i][2] as f32 / 255.0,
                1.0,
            ];

            let column_line_width1 = self.config.column_line_width[i] as f64 * stage_h / 1024.0; // wtaf theres a 3rd scale???
            let column_line_width2 = self.config.column_line_width[i+1] as f64 * stage_h / 1024.0;
            let alpha = (self.config.colour_column_line[3] as f32 / 255.0).powf(2.0); // wtf???
            let column_line_color = [
                self.config.colour_column_line[0] as f32 / 255.0 * alpha,
                self.config.colour_column_line[1] as f32 / 255.0 * alpha,
                self.config.colour_column_line[2] as f32 / 255.0 * alpha,
                self.config.colour_column_line[3] as f32 / 255.0,
            ];
            let column_line_rect1 = [
                key_x,
                0.0,
                column_line_width1,
                hit_position,
            ];
            let column_line_rect2 = [
                key_x + key_width,
                0.0,
                column_line_width2,
                hit_position,
            ];
            let column_line_rect = graphics::Rectangle::new(column_line_color);
            let ds = draw_state.blend(draw_state::Blend::Add);
            column_line_rect.draw(column_line_rect1, &ds, transform, g);
            column_line_rect.draw(column_line_rect2, &ds, transform, g);

            let sl_size = self.textures.stage_light.len();
            let stage_light_height =
                self.textures.stage_light[sl_size - 1].get_height() as f64 * scale2;

            let stage_light_y = self.config.light_position as f64 * scale - stage_light_height;

            let stage_light_img = Image::new()
                .rect([
                    key_x,
                    stage_light_y,
                    key_width,
                    stage_light_height,
                ]);

            if let Some(last_down_time) = self.anim_states.keys_last_down_time[i] {
                let current_time = time::Instant::now();
                let elapsed_time = current_time - last_down_time;
                let elapsed_time_secs = elapsed_time.as_secs() as f64
                    + elapsed_time.subsec_nanos() as f64 / 1e9;
                let fframe: f32 = elapsed_time_secs as f32 * 30.0;
                let frame = fframe as usize;
                if frame < 3 {
                    color[3] -= fframe / 3.0;
                    stage_light_img.color(color).draw(
                        self.textures.stage_light[sl_size - 1].as_ref(),
                        draw_state,
                        transform,
                        g,
                    );
                }
            } else if *key_pressed {
                stage_light_img.color(color).draw(
                    self.textures.stage_light[0].as_ref(),
                    draw_state,
                    transform,
                    g,
                );
            }
            key_img.draw(key_texture, draw_state, transform, g);
        }
    }

    fn draw_perfect(
        &self,
        draw_state: &DrawState,
        transform: math::Matrix2d,
        size_scale: f64,
        g: &mut G,
        stage_h: f64,
        elapsed_time: time::Duration,
    ) {
        let elapsed =
            elapsed_time.as_secs() as f64 + elapsed_time.subsec_nanos() as f64 / 1e9;
        let frame = (elapsed * 30.0) as usize % self.textures.hit300g.len();

        let tx = &*self.textures.hit300g[frame];

        let scale = stage_h / 480.0;
        let scale2 = stage_h / 768.0;
        let stage_width = (self.config.column_width.iter().sum::<u16>() as f64
            + self.config.column_spacing.iter().sum::<u16>() as f64)
            * scale;
        let column_start = self.config.column_start as f64 * scale;

        let tx_w = tx.get_width() as f64 * scale2 / 1.5 * size_scale;
        let tx_h = tx.get_height() as f64 * scale2 / 1.5 * size_scale;
        let tx_x = stage_width / 2.0 - tx_w / 2.0 + column_start;
        let tx_y = self.config.score_position as f64 * scale - tx_h / 2.0;

        let img = Image::new().rect([tx_x, tx_y, tx_w, tx_h]);
        img.draw(tx, draw_state, transform, g);
    }

    fn draw_miss(
        &self,
        draw_state: &DrawState,
        transform: math::Matrix2d,
        g: &mut G,
        stage_h: f64,
    ) {
        let tx = &*self.textures.miss[0];

        let scale = stage_h / 480.0;
        let scale2 = stage_h / 768.0;
        let stage_width = (self.config.column_width.iter().sum::<u16>() as f64
            + self.config.column_spacing.iter().sum::<u16>() as f64)
            * scale;
        let column_start = self.config.column_start as f64 * scale;

        let tx_w = tx.get_width() as f64 * scale2;
        let tx_h = tx.get_height() as f64 * scale2;
        let tx_x = stage_width / 2.0 - tx_w / 2.0 + column_start;
        let tx_y = self.config.score_position as f64 * scale - tx_h / 2.0;

        let img = Image::new().rect([tx_x, tx_y, tx_w, tx_h]);
        img.draw(tx, draw_state, transform, g);
    }

    fn draw_hit_anims(
        &mut self,
        draw_state: &DrawState,
        transform: math::Matrix2d,
        g: &mut G,
        stage_height: f64,
    ) {
        let scale = stage_height / 480.0;
        let scale2 = stage_height / 768.0;

        let hit_p = self.config.hit_position as f64 * scale;

        for (i, hit_anim) in self.anim_states.hit_anim.iter_mut().enumerate() {
            let key_width = self.config.column_width[i] as f64 * scale;
            let hit_x = scale
                * (self.config.column_start as f64
                    + self.config.column_width[0..i].iter().sum::<u16>() as f64
                    + self.config.column_spacing[0..i].iter().sum::<u16>() as f64);

            match *hit_anim {
                HitAnimState::SingleNote(time) => {
                    let frame = (time.elapsed() * 60).as_secs() as usize;
                    if frame > self.textures.lighting_n.len() - 1 {
                        *hit_anim = HitAnimState::None;
                    } else {
                        let hit_w = self.textures.lighting_n[frame].get_width() as f64 * scale2;
                        let hit_h = self.textures.lighting_n[frame].get_height() as f64 * scale2;
                        let hit_img = Image::new().rect([
                            hit_x - hit_w / 2.0 + key_width / 2.0,
                            hit_p - hit_h / 2.0,
                            hit_w,
                            hit_h,
                        ]);
                        hit_img.draw(
                            &*self.textures.lighting_n[frame],
                            draw_state,
                            transform,
                            g,
                        );
                    }
                }
                HitAnimState::LongNote(time) => {
                    let frame = (time.elapsed() * 60).as_secs() as usize % self.textures.lighting_l.len();
                    let hit_w = self.textures.lighting_l[frame].get_width() as f64 * scale2;
                    let hit_h = self.textures.lighting_l[frame].get_height() as f64 * scale2;
                    let hit_img = Image::new().rect([
                        hit_x - hit_w / 2.0 + key_width / 2.0,
                        hit_p - hit_h / 2.0,
                        hit_w,
                        hit_h,
                    ]);
                    hit_img.draw(
                        &*self.textures.lighting_l[frame],
                        draw_state,
                        transform,
                        g,
                    );
                }
                HitAnimState::LongNoteFinal(start, end) => {
                    let diff = end - start;
                    let elapsed = start.elapsed();

                    let anim_count1 = (diff * 60 / self.textures.lighting_l.len() as u32).as_secs();
                    let anim_count2 = (elapsed * 60 / self.textures.lighting_l.len() as u32).as_secs();

                    if anim_count2 > anim_count1 {
                        *hit_anim = HitAnimState::None;
                    } else {
                        let frame = (elapsed * 60).as_secs() as usize % self.textures.lighting_l.len();
                        let hit_w = self.textures.lighting_l[frame].get_width() as f64 * scale2;
                        let hit_h = self.textures.lighting_l[frame].get_height() as f64 * scale2;
                        let hit_img = Image::new().rect([
                            hit_x - hit_w / 2.0 + key_width / 2.0,
                            hit_p - hit_h / 2.0,
                            hit_w,
                            hit_h,
                        ]);
                        hit_img.draw(
                            &*self.textures.lighting_l[frame],
                            draw_state,
                            transform,
                            g,
                        );
                    }
                }
                HitAnimState::None => (),
            }
        }
    }
}

#[derive(Debug)]
enum OsuSkinParseError {
    NoDefaultTexture(String),
}

impl fmt::Display for OsuSkinParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            OsuSkinParseError::NoDefaultTexture(ref s) => {
                write!(f, "No default texture found for {}", s)
            }
        }
    }
}

impl From<OsuSkinParseError> for ParseError {
    fn from(e: OsuSkinParseError) -> ParseError {
        ParseError::Parse(String::from("Error reading osu skin"), Some(Box::new(e)))
    }
}

impl error::Error for OsuSkinParseError {
    fn description(&self) -> &str {
        match *self {
            OsuSkinParseError::NoDefaultTexture(_) => "No default texture found",
        }
    }
}

// Apparently I need to do this?
fn fix_alpha(img: &mut image::RgbaImage) {
    use std::u8;

    // linearize the alpha channel (wtf)
    for pixel in img.pixels_mut() {
        const U8_MAX: f32 = u8::MAX as f32;

        let mut v = pixel.0[3] as f32 / U8_MAX;

        if v <= 0.04045 {
            v /= 12.92
        } else {
            v = ((v + 0.055) / 1.055).powf(2.4)
        }

        pixel.0[3] = (v * U8_MAX).round() as u8;
    }
}

fn texture_from_path<F, T, P>(
    factory: &mut F,
    path: P,
    texture_settings: &TextureSettings,
) -> Result<T, ParseError>
where
    T: CreateTexture<F>,
    P: AsRef<path::Path>,
    T::Error: ToString,
{
    let path_string = path.as_ref().to_string_lossy().into_owned();
    let mut image = match image::open(&path) {
        Ok(t) => t.to_rgba8(),
        Err(e) => return Err(ParseError::ImageError(path_string, e)),
    };
    fix_alpha(&mut image); // ???
    let dimensions = image.dimensions();
    CreateTexture::create(
        factory,
        Format::Rgba8,
        &*image.into_raw(),
        [dimensions.0, dimensions.1],
        texture_settings,
    ).map_err(|e: T::Error| {
        ParseError::TextureError {
            path: path.as_ref().to_owned(),
            error: e.to_string(),
        }
    })
}

/// Load an animatable skin element's textures
///
/// This function takes the basename and tries different paths until it finds one that exists
fn load_texture_anim<F, T>(
    factory: &mut F,
    cache: &mut HashMap<String, Rc<[Rc<T>]>>,
    dir: &path::Path,
    default_dir: &path::Path,
    names: &(&'static str, String),
    texture_settings: &TextureSettings,
) -> Result<Rc<[Rc<T>]>, ParseError>
where
    T: CreateTexture<F>,
    T::Error: ToString,
{
    let mut textures = Vec::new();
    let mut path;

    macro_rules! repetitive_code {
        // $dir should be a path::Path
        ($(($dir:ident, $name:expr)),*) => {$(

            // Check the cache
            if let Some(texture) = cache.get(&$name) {
                return Ok(Rc::clone(texture));
            }

            // TODO can these join's be optimized? how much time does it take to allocate the pathbuf?

            // Check for an animation sequence
            path = $dir.join($name + "-0.png");
            if path.exists() {
                textures.push(Rc::new(texture_from_path(factory, &path, texture_settings)?));
                let mut n = 1;
                loop {
                    path = $dir.join(format!("{}-{}.png", $name, n));
                    if !path.exists() { break; }
                    textures.push(Rc::new(texture_from_path(factory, &path, texture_settings)?));
                    n += 1;
                }
                let anim = Rc::from(textures);
                cache.insert($name, Rc::clone(&anim));
                return Ok(anim);
            }

            // Check for static image
            path = $dir.join($name + ".png");
            if path.exists() {
                // help
                let texture = Rc::new(texture_from_path(factory, &path, texture_settings)?);
                let anim = Rc::from(&[texture][..]);
                cache.insert($name, Rc::clone(&anim));
                return Ok(anim);
            }
        )*}
    }

    // Check the skin directory, then the default skin directory
    repetitive_code!((dir, names.1.clone()), (default_dir, names.0.to_owned()));

    Err(OsuSkinParseError::NoDefaultTexture(String::from(names.0)).into())
}

/// Load a skin element's texture
///
/// This function takes the basename and tries different paths until it finds one that exists
fn load_texture<F, T>(
    factory: &mut F,
    cache: &mut HashMap<String, Rc<[Rc<T>]>>,
    dir: &path::Path,
    default_dir: &path::Path,
    names: &(&'static str, String),
    texture_settings: &TextureSettings,
) -> Result<Rc<T>, ParseError>
where
    T: CreateTexture<F>,
    T::Error: ToString,
{
    macro_rules! repetitive_code {
        // $dir should be a path::Path
        ($(($dir:ident, $name:expr)),*) => {$(

            // Check the cache
            if let Some(texture) = cache.get(&$name) {
                return Ok(Rc::clone(&texture[0]));
            }

            // TODO can these join's be optimized? how much time does it take to allocate the pathbuf?
            let path = $dir.join($name + ".png");
            if path.exists() {
                let texture = texture_from_path(factory, path, texture_settings)?;
                let rc = Rc::new(texture);
                cache.insert($name, Rc::from(&[Rc::clone(&rc)][..]));
                return Ok(rc);
            }
        )*}
    }

    // Check the skin directory, then the default skin directory
    repetitive_code!((dir, names.1.clone()), (default_dir, names.0.to_owned()));

    Err(OsuSkinParseError::NoDefaultTexture(String::from(names.0)).into())
}

pub fn from_path<F, G>(
    factory: &mut F,
    dir: &path::Path,
    default_dir: &path::Path,
) -> Result<Box<dyn GameSkin<G>>, ParseError>
where
    G: Graphics + 'static,
    G::Texture: CreateTexture<F>,
    <G::Texture as TextureOp<F>>::Error: ToString,
{
    let config_path = dir.join(path::Path::new("skin.ini"));

    let texture_settings = TextureSettings::new();

    macro_rules! double {
        ($e:expr) => {
            ($e, String::from($e))
        };
    }

    // put things into the 1213121 pattern
    macro_rules! pat {
        ($a:expr, $b:expr, $c:expr) => {
            [$a, $b, $a, $c, $a, $b, $a]
        };
    }

    // (default image name, skin image name)
    // the skin filename might get changed by the skin.ini, which is parsed later
    let mut miss_name = double!("mania-hit0");
    let mut hit50_name = double!("mania-hit50");
    let mut hit100_name = double!("mania-hit100");
    let mut hit200_name = double!("mania-hit200");
    let mut hit300_name = double!("mania-hit300");
    let mut hit300g_name = double!("mania-hit300g");
    let mut stage_light_name = double!("mania-stage-light");
    let mut lighting_n_name = double!("lightingN");
    let mut lighting_l_name = double!("lightingL");

    let mut keys_name = pat![
        double!("mania-key1"),
        double!("mania-key2"),
        double!("mania-keyS")
    ];

    let mut keys_d_name = pat![
        double!("mania-key1D"),
        double!("mania-key2D"),
        double!("mania-keySD")
    ];

    let mut notes_name = pat![
        double!("mania-note1"),
        double!("mania-note2"),
        double!("mania-noteS")
    ];

    // lns is plural of ln (long note)
    let mut lns_head_name = pat![
        double!("mania-note1H"),
        double!("mania-note2H"),
        double!("mania-noteSH")
    ];

    let mut lns_body_name = pat![
        double!("mania-note1L"),
        double!("mania-note2L"),
        double!("mania-noteSL")
    ];

    let mut lns_tail_name = pat![
        double!("mania-note1T"),
        double!("mania-note2T"),
        double!("mania-noteST")
    ];

    let mut stage_hint_name = double!("mania-stage-hint");
    let mut stage_left_name = double!("mania-stage-left");
    let mut stage_right_name = double!("mania-stage-right");
    let mut stage_bottom_name = double!("mania-stage-bottom");

    // default values
    let mut column_start = 136;
    let mut column_width = [30; 7];
    let mut column_line_width = [2; 8];
    let mut colour_column_line = [255; 4];
    let mut column_spacing = [0; 6];
    let mut colour_light = [[255, 255, 255]; 7];
    let mut hit_position = 402;
    let mut score_position = 240; // idk TODO
    let mut light_position = 413;
    let mut note_body_style = [NoteBodyStyle::CascadeFromTop; 7];

    // parse skin.ini
    if config_path.exists() {
        let config_file = File::open(config_path)
            .map_err(|e| ParseError::Io(String::from("Error opening config file"), e))?;
        let config_reader = BufReader::new(&config_file);
        let mut section = String::from("General");
        let mut keys: u8 = 0;
        for (line_number, l) in config_reader.lines().enumerate().map(|(n, l)| (n + 1, l)) {
            let line =
                l.map_err(|e| ParseError::Io(String::from("Error reading config file"), e))?;
            let line = line.trim();

            // section declarations look like [section name]
            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len() - 1].to_string();
                continue;
            }

            // comment line or empty line
            if line.starts_with("//") || line == "" {
                continue;
            }

            // key: value
            let mut line_parts = line.splitn(2, ':');

            // parse but with some error handling
            macro_rules! parse {
                ($value:ident) => {
                    match $value.parse() {
                        Ok(o) => o,
                        Err(e) => {
                            remani_warn!(
                                "Malformed value in line {} of skin.ini ({}), ignoring",
                                line_number,
                                e
                            );
                            continue;
                        }
                    }
                };
            }

            let key = if let Some(k) = line_parts.next() {
                k.trim()
            } else {
                continue;
            };
            let value = if let Some(v) = line_parts.next() {
                v.trim()
            } else {
                continue;
            };
            match key {
                "Keys" => keys = parse!(value),
                _ => {
                    if keys == 7 && section == "Mania" {
                        macro_rules! prop_name {
                            ({$prefix:ident#$suffix:ident}, $n:expr) => {
                                concat!(
                                    concat!(stringify!($prefix), stringify!($n)),
                                    stringify!($suffix)
                                )
                            };
                            ({$prefix:ident#}, $n:expr) => {
                                concat!(stringify!($prefix), stringify!($n))
                            };
                        }
                        // fancy macros
                        // used to match stuff like KeyImage{0..6}H more easily
                        macro_rules! enumerate_match {
                            ($key:ident,
                             $(.$name1:tt => $varname1:ident = $value1:expr, [ $baseidx1:literal $($idx1:literal)* ],)*
                             ==
                             $(.$name2:tt => $varname2:ident = $value2:expr, [ $baseidx2:literal $($idx2:literal)* ],)*) => {
                                match $key {
                                    $(
                                        prop_name!($name1, $baseidx1) => $varname1[0].1 = $value1,
                                        $(prop_name!($name1, $idx1) => $varname1[$idx1 - $baseidx1].1 = $value1,)*
                                    )*

                                    $(
                                        prop_name!($name2, $baseidx2) => $varname2[0] = $value2,
                                        $(prop_name!($name2, $idx2) => $varname2[$idx2 - $baseidx2] = $value2,)*
                                    )*
                                    _ => (),
                                }
                            };
                        }

                        // for values that look like
                        // 42,10,5,1337,4,8,2
                        macro_rules! csv {
                            ($default:expr; $count:expr) => {{
                                let mut a = $default;
                                let mut n = 0;
                                for (i, v) in value.split(",").enumerate().take($count) {
                                    a[i] = parse!(v);
                                    n = i;
                                }
                                if n < $count - 1 {
                                    remani_warn!(
                                        "Malformed value in line {} of skin.ini (not enough fields), ignoring",
                                        line_number
                                    );
                                    continue;
                                } else {
                                    a
                                }
                            }};
                        }
                        match key {
                            "ColumnStart" => column_start = parse!(value),
                            "HitPosition" => hit_position = parse!(value),
                            "ScorePosition" => score_position = parse!(value),
                            "LightPosition" => light_position = parse!(value),
                            "ColumnWidth" => column_width = csv![column_width; 7],
                            "ColumnLineWidth" => column_line_width = csv![column_line_width; 8],
                            "ColourColumnLine" => colour_column_line = csv![colour_column_line; 4],
                            "ColumnSpacing" => column_spacing = csv![column_spacing; 6],
                            "NoteBodyStyle" => note_body_style = [parse!(value); 7],
                            "Hit0" => miss_name.1 = value.to_owned(),
                            "Hit50" => hit50_name.1 = value.to_owned(),
                            "Hit100" => hit100_name.1 = value.to_owned(),
                            "Hit200" => hit200_name.1 = value.to_owned(),
                            "Hit300" => hit300_name.1 = value.to_owned(),
                            "Hit300g" => hit300g_name.1 = value.to_owned(),
                            "StageHint" => stage_hint_name.1 = value.to_owned(),
                            "StageLeft" => stage_left_name.1 = value.to_owned(),
                            "StageRight" => stage_right_name.1 = value.to_owned(),
                            "StageBottom" => stage_bottom_name.1 = value.to_owned(),
                            "StageLight" => stage_light_name.1 = value.to_owned(),
                            "LightingN" => lighting_n_name.1 = value.to_owned(),
                            "LightingL" => lighting_l_name.1 = value.to_owned(),

                            k => enumerate_match! { k,
                                .{KeyImage#} => keys_name = value.to_owned(), [0 1 2 3 4 5 6],
                                .{KeyImage#D} => keys_d_name = value.to_owned(), [0 1 2 3 4 5 6],
                                .{NoteImage#} => notes_name = value.to_owned(), [0 1 2 3 4 5 6],
                                .{NoteImage#H} => lns_head_name = value.to_owned(), [0 1 2 3 4 5 6],
                                .{NoteImage#L} => lns_body_name = value.to_owned(), [0 1 2 3 4 5 6],
                                .{NoteImage#T} => lns_tail_name = value.to_owned(), [0 1 2 3 4 5 6],
                                == // separator between the image file specifying properties above, and the other properties below
                                .{ColourLight#} => colour_light = csv![[0; 3]; 3], [1 2 3 4 5 6 7],
                                .{NoteBodyStyle#} => note_body_style = parse!(value), [0 1 2 3 4 5 6],
                            },
                        }
                    }
                }
            }
        }
    }

    let mut cache = HashMap::new();

    // load all the textures into the gpu

    let miss = load_texture_anim(factory, &mut cache, dir, default_dir, &miss_name, &texture_settings)?;
    let hit50 = load_texture_anim(factory, &mut cache, dir, default_dir, &hit50_name, &texture_settings)?;
    let hit100 = load_texture_anim(factory, &mut cache, dir, default_dir, &hit100_name, &texture_settings)?;
    let hit200 = load_texture_anim(factory, &mut cache, dir, default_dir, &hit200_name, &texture_settings)?;
    let hit300 = load_texture_anim(factory, &mut cache, dir, default_dir, &hit300_name, &texture_settings)?;
    let hit300g = load_texture_anim(factory, &mut cache, dir, default_dir, &hit300g_name, &texture_settings)?;
    let stage_light = load_texture_anim(factory, &mut cache, dir, default_dir, &stage_light_name, &texture_settings)?;
    let lighting_n = load_texture_anim(factory, &mut cache, dir, default_dir, &lighting_n_name, &texture_settings)?;
    let lighting_l = load_texture_anim(factory, &mut cache, dir, default_dir, &lighting_l_name, &texture_settings)?;
    let keys = [load_texture(factory, &mut cache, dir, default_dir, &keys_name[0], &texture_settings)?,
                load_texture(factory, &mut cache, dir, default_dir, &keys_name[1], &texture_settings)?,
                load_texture(factory, &mut cache, dir, default_dir, &keys_name[2], &texture_settings)?,
                load_texture(factory, &mut cache, dir, default_dir, &keys_name[3], &texture_settings)?,
                load_texture(factory, &mut cache, dir, default_dir, &keys_name[4], &texture_settings)?,
                load_texture(factory, &mut cache, dir, default_dir, &keys_name[5], &texture_settings)?,
                load_texture(factory, &mut cache, dir, default_dir, &keys_name[6], &texture_settings)?];

    let keys_d = [load_texture(factory, &mut cache, dir, default_dir, &keys_d_name[0], &texture_settings)?,
                  load_texture(factory, &mut cache, dir, default_dir, &keys_d_name[1], &texture_settings)?,
                  load_texture(factory, &mut cache, dir, default_dir, &keys_d_name[2], &texture_settings)?,
                  load_texture(factory, &mut cache, dir, default_dir, &keys_d_name[3], &texture_settings)?,
                  load_texture(factory, &mut cache, dir, default_dir, &keys_d_name[4], &texture_settings)?,
                  load_texture(factory, &mut cache, dir, default_dir, &keys_d_name[5], &texture_settings)?,
                  load_texture(factory, &mut cache, dir, default_dir, &keys_d_name[6], &texture_settings)?];

    let notes = [load_texture_anim(factory, &mut cache, dir, default_dir, &notes_name[0], &texture_settings)?,
                 load_texture_anim(factory, &mut cache, dir, default_dir, &notes_name[1], &texture_settings)?,
                 load_texture_anim(factory, &mut cache, dir, default_dir, &notes_name[2], &texture_settings)?,
                 load_texture_anim(factory, &mut cache, dir, default_dir, &notes_name[3], &texture_settings)?,
                 load_texture_anim(factory, &mut cache, dir, default_dir, &notes_name[4], &texture_settings)?,
                 load_texture_anim(factory, &mut cache, dir, default_dir, &notes_name[5], &texture_settings)?,
                 load_texture_anim(factory, &mut cache, dir, default_dir, &notes_name[6], &texture_settings)?];

    let long_notes_head = [load_texture_anim(factory, &mut cache, dir, default_dir, &lns_head_name[0], &texture_settings)?,
                           load_texture_anim(factory, &mut cache, dir, default_dir, &lns_head_name[1], &texture_settings)?,
                           load_texture_anim(factory, &mut cache, dir, default_dir, &lns_head_name[2], &texture_settings)?,
                           load_texture_anim(factory, &mut cache, dir, default_dir, &lns_head_name[3], &texture_settings)?,
                           load_texture_anim(factory, &mut cache, dir, default_dir, &lns_head_name[4], &texture_settings)?,
                           load_texture_anim(factory, &mut cache, dir, default_dir, &lns_head_name[5], &texture_settings)?,
                           load_texture_anim(factory, &mut cache, dir, default_dir, &lns_head_name[6], &texture_settings)?];

    let long_notes_body = [load_texture_anim(factory, &mut cache, dir, default_dir, &lns_body_name[0], &texture_settings)?,
                           load_texture_anim(factory, &mut cache, dir, default_dir, &lns_body_name[1], &texture_settings)?,
                           load_texture_anim(factory, &mut cache, dir, default_dir, &lns_body_name[2], &texture_settings)?,
                           load_texture_anim(factory, &mut cache, dir, default_dir, &lns_body_name[3], &texture_settings)?,
                           load_texture_anim(factory, &mut cache, dir, default_dir, &lns_body_name[4], &texture_settings)?,
                           load_texture_anim(factory, &mut cache, dir, default_dir, &lns_body_name[5], &texture_settings)?,
                           load_texture_anim(factory, &mut cache, dir, default_dir, &lns_body_name[6], &texture_settings)?];

    let long_notes_tail = [load_texture_anim(factory, &mut cache, dir, default_dir, &lns_tail_name[0], &texture_settings).ok(),
                           load_texture_anim(factory, &mut cache, dir, default_dir, &lns_tail_name[1], &texture_settings).ok(),
                           load_texture_anim(factory, &mut cache, dir, default_dir, &lns_tail_name[2], &texture_settings).ok(),
                           load_texture_anim(factory, &mut cache, dir, default_dir, &lns_tail_name[3], &texture_settings).ok(),
                           load_texture_anim(factory, &mut cache, dir, default_dir, &lns_tail_name[4], &texture_settings).ok(),
                           load_texture_anim(factory, &mut cache, dir, default_dir, &lns_tail_name[5], &texture_settings).ok(),
                           load_texture_anim(factory, &mut cache, dir, default_dir, &lns_tail_name[6], &texture_settings).ok()];

    let stage_hint = load_texture_anim(factory, &mut cache, dir, default_dir, &stage_hint_name, &texture_settings)?;
    let stage_left = load_texture(factory, &mut cache, dir, default_dir, &stage_left_name, &texture_settings)?;
    let stage_right = load_texture(factory, &mut cache, dir, default_dir, &stage_right_name, &texture_settings)?;
    let stage_bottom = load_texture_anim(factory, &mut cache, dir, default_dir, &stage_bottom_name, &texture_settings).ok();

    let smallest_note_width;
    let smallest_note_height;
    {
        let smallest_height_note = &notes
            .iter()
            .min_by_key(|x: &&Rc<[Rc<G::Texture>]>| x[0].get_height())
            .unwrap()[0];
        smallest_note_width = smallest_height_note.get_width() as f64;
        smallest_note_height = smallest_height_note.get_height() as f64;
    }
    let width_for_note_height_scale =
        smallest_note_height / smallest_note_width * *column_width.iter().min().unwrap() as f64;
    Ok(Box::new(OsuSkin {
        textures: OsuSkinTextures {
            miss,
            hit50,
            hit100,
            hit200,
            hit300,
            hit300g,
            stage_light,
            lighting_n,
            lighting_l,
            keys,
            keys_d,
            notes,
            long_notes_head,
            long_notes_body,
            long_notes_tail,
            stage_hint,
            stage_left,
            stage_right,
            stage_bottom,
        },

        anim_states: OsuAnimStates {
            keys_last_down_time: [None; 7],
            hit_anim: [HitAnimState::None; 7],
        },

        config: OsuSkinConfig {
            column_start,
            column_width,
            column_spacing,
            column_line_width,
            colour_column_line,
            hit_position,
            score_position,
            light_position,
            width_for_note_height_scale,
            note_body_style,
            colour_light,
        },
        judgement: None,
    }))
}
