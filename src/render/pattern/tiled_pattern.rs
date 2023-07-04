use std::rc::Rc;
use pdf_writer::{Content, Finish, PdfWriter};
use usvg::{NodeKind, Size, Transform, Units};
use usvg::utils::view_box_to_transform;
use pdf_writer::types::{PaintType, TilingType};
use crate::render::group::create_x_object;
use crate::util::context::Context;
use crate::util::helper::{NameExt, TransformExt};

pub fn create(
    pattern: Rc<usvg::Pattern>,
    parent_bbox: &usvg::Rect,
    writer: &mut PdfWriter,
    ctx: &mut Context,
) -> String {
    let (pattern_name, pattern_id) = ctx.deferrer.add_pattern();
    ctx.deferrer.push();

    // Content units object bounding box should only be used if no viewbox is declared.
    let use_content_units_obb =
        pattern.content_units == Units::ObjectBoundingBox && pattern.view_box.is_none();

    let pattern_rect =
        if pattern.units == Units::ObjectBoundingBox || use_content_units_obb {
            pattern.rect.bbox_transform(*parent_bbox)
        } else {
            pattern.rect
        };

    match *pattern.root.borrow() {
        NodeKind::Group(ref group) => {
            let mut pattern_matrix = ctx.context_frame.full_transform();
            pattern_matrix.append(&pattern.transform);
            // Make sure that the pattern moves accordingly when a different x/y value is set for the pattern
            pattern_matrix.append(&Transform::new(
                1.0,
                0.0,
                0.0,
                1.0,
                pattern_rect.x(),
                pattern_rect.y(),
            ));
            // All transformations up until now will be applied to the pattern by setting the matrix argument of the pattern,
            // so we create a completely new context frame here which doesn't contain any of the transformations up until now
            ctx.context_frame.push_new();

            if use_content_units_obb {
                // Again, the x/y is already accounted for in the pattern matrix, so we only need to scale the height/width. Otherwise,
                // the x/y would be applied twice.
                ctx.context_frame.append_transform(&Transform::new_scale(
                    parent_bbox.width(),
                    parent_bbox.height(),
                ));
            }

            if let Some(view_box) = pattern.view_box {
                ctx.context_frame.append_transform(&view_box_to_transform(
                    view_box.rect,
                    view_box.aspect,
                    Size::new(pattern_rect.width(), pattern_rect.height()).unwrap(),
                ));
            }

            let (x_object_name, _) = create_x_object(&pattern.root, group, writer, ctx);

            let mut pattern_content = Content::new();
            pattern_content.x_object(x_object_name.as_name());
            let pattern_content_stream = pattern_content.finish();

            let mut tiling_pattern =
                writer.tiling_pattern(pattern_id, &pattern_content_stream);

            let mut resources = tiling_pattern.resources();
            ctx.deferrer.pop(&mut resources);
            resources.finish();

            // We already account for the x/y of the pattern by appending it to the matrix above, so here we just need to take the height / width
            // in consideration
            let final_bbox = pdf_writer::Rect::new(
                0.0,
                0.0,
                pattern_rect.width() as f32,
                pattern_rect.height() as f32,
            );

            tiling_pattern
                .tiling_type(TilingType::ConstantSpacing)
                .paint_type(PaintType::Colored)
                .bbox(final_bbox)
                .matrix(pattern_matrix.as_array())
                .x_step(final_bbox.x2 - final_bbox.x1)
                .y_step(final_bbox.y2 - final_bbox.y1);

            ctx.context_frame.pop();
            pattern_name
        }
        _ => unreachable!(),
    }
}
