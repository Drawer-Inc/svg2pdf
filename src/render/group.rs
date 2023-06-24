use crate::util::Context;
use crate::render::clip;
use crate::render::Render;
use pdf_writer::{Content, Finish, PdfWriter, Ref};
use usvg::Node;
use crate::util::helper::NameExt;

pub(crate) fn render(
    node: &Node,
    group: &usvg::Group,
    writer: &mut PdfWriter,
    content: &mut Content,
    ctx: &mut Context,
) {
    let (name, _) = create_x_object(node, group, writer, ctx);
    content.x_object(name.as_name());
}

pub(crate) fn create_x_object(
    node: &Node,
    group: &usvg::Group,
    writer: &mut PdfWriter,
    ctx: &mut Context,
) -> (String, Ref) {
    let (name, reference) = ctx.deferrer.add_x_object();

    ctx.deferrer.push_context();

    let mut child_content = Content::new();

    child_content.save_state();

    ctx.context_frame.push();
    ctx.context_frame.append_transform(&group.transform);

    let pdf_bbox = ctx.pdf_bbox(node);

    if let Some(clip_path) = &group.clip_path {
        let name = clip::render(node, clip_path.clone(), writer, ctx);
        child_content.set_parameters(name.as_name());
    }

    if group.opacity.get() != 1.0 {
        let name = ctx.deferrer.add_opacity(None, Some(group.opacity.get() as f32));
        child_content.set_parameters(name.as_name());
    }

    for child in node.children() {
        child.render(writer, &mut child_content, ctx);
    }

    ctx.context_frame.pop();
    child_content.restore_state();

    let child_content_stream = child_content.finish();

    let mut x_object = writer.form_xobject(reference, &child_content_stream);
    ctx.deferrer.pop_context(&mut x_object.resources());

    let mut group = x_object.group();
    group
        .transparency()
        .isolated(true)
        .knockout(false)
        .color_space()
        .srgb();
    group.finish();

    x_object.bbox(pdf_bbox);
    x_object.finish();
    (name, reference)
}
