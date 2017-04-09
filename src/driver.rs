// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use ast::Idents;
use cairo::{Cairo, FontFace};
use elements::{Color, Element, PlacedElement};
use runtime::{FontMap, Frame};

fn draw_background(cr: &mut Cairo, color: Color) {
    // TODO: Do not hard-code the canvas dimensions.
    cr.rectangle(0.0, 0.0, 1920.0, 1080.0);
    cr.set_source_rgb(color.r, color.g, color.b);
    cr.fill();
}

fn draw_element(fm: &mut FontMap, cr: &mut Cairo, pe: &PlacedElement) {
    match pe.element {
        Element::Line(ref line) => {
            cr.move_to(pe.position.x, pe.position.y);
            cr.set_source_rgb(line.color.r, line.color.g, line.color.b);
            cr.set_line_width(line.line_width);
            let to = pe.position + line.offset;
            cr.line_to(to.x, to.y);
            cr.stroke();
        }

        Element::Text(ref text) => {
            // Cairo uses absolute positions for glyphs, so we need to add
            // the final positions to the glyph locations.
            let glyphs_offset: Vec<_> = text.glyphs.iter()
                                            // TODO: Make offset type take
                                            // Vec2.
                                            .map(|g| g.offset(pe.position.x, pe.position.y))
                                            .collect();
            // If we were able to shape the text, then the FT font must
            // exist still. TODO: Would it be better to just embed a
            // reference in the Text element instead of doing the lookup
            // twice?
            let ft_face = fm.get(&text.font_family, &text.font_style).unwrap();
            let cr_face = FontFace::from_ft_face(ft_face.clone());
            cr.set_font_face(&cr_face);
            cr.set_font_size(text.font_size);
            cr.set_source_rgb(text.color.r, text.color.g, text.color.b);
            cr.show_glyphs(&glyphs_offset);
            // TODO: The cr_font should outlive the Cairo, because Cairo
            // might internally reference the font still. How to model this?
        }

        Element::Scaled(ref elements, scale) => {
            // Store the current transform so we can restore it later.
            let matrix = cr.get_matrix();
            cr.translate(pe.position.x, pe.position.y);
            cr.scale(scale, scale);
            for inner_pe in elements {
                draw_element(fm, cr, inner_pe);
            }
            cr.set_matrix(&matrix);
        }

        Element::Svg(ref svg) => {
            // Store the current transform so we can restore it later.
            let matrix = cr.get_matrix();
            cr.translate(pe.position.x, pe.position.y);
            svg.draw(cr);
            cr.set_matrix(&matrix);
        }
    }
}

pub fn render_frame<'a>(fm: &mut FontMap, cr: &mut Cairo, frame: &Frame<'a>) {
    // TODO: Ensure that writing to background_color only accepts a color value,
    // so a lookup failure here is never a type error.
    let var_bgcolor = Idents(vec!["background_color"]);
    if let Ok(bgcolor) = frame.get_env().lookup_color(&var_bgcolor) {
        draw_background(cr, bgcolor);
    }

    for pe in frame.get_elements() {
        draw_element(fm, cr, pe);
    }
    cr.show_page()
}
