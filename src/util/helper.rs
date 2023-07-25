use pdf_writer::types::BlendMode;
use pdf_writer::{Name, Rect};
use usvg::{BBox, Node, NodeExt, NodeKind, NonZeroRect, Size, Transform};

pub const SRGB: Name = Name(b"srgb");

/// Extension trait to convert [Colors](usvg::Color) into PDF colors.
pub trait ColorExt {
    fn as_array(&self) -> [f32; 3];
}

impl ColorExt for usvg::Color {
    fn as_array(&self) -> [f32; 3] {
        [self.red as f32 / 255.0, self.green as f32 / 255.0, self.blue as f32 / 255.0]
    }
}

/// Extension trait to convert a [Transform] into PDF transforms.
pub trait TransformExt {
    fn as_array(&self) -> [f32; 6];
}

impl TransformExt for Transform {
    fn as_array(&self) -> [f32; 6] {
        [self.sx, self.ky, self.kx, self.sy, self.tx, self.ty]
    }
}

/// Extension trait to convert a [String] into a [Name]
pub trait NameExt {
    fn as_name(&self) -> Name;
}

impl NameExt for String {
    fn as_name(&self) -> Name {
        Name(self.as_bytes())
    }
}

/// Extension trait to turn a [`usvg` Rect](usvg::Rect) into a [PDF Rect](Rect)
pub trait RectExt {
    fn as_pdf_rect(&self) -> Rect;
}

impl RectExt for NonZeroRect {
    fn as_pdf_rect(&self) -> Rect {
        Rect::new(self.x(), self.y(), self.x() + self.width(), self.y() + self.height())
    }
}

pub trait BlendModeExt {
    fn to_pdf_blend_mode(&self) -> BlendMode;
}

impl BlendModeExt for usvg::BlendMode {
    fn to_pdf_blend_mode(&self) -> BlendMode {
        match self {
            usvg::BlendMode::Normal => BlendMode::Normal,
            usvg::BlendMode::Multiply => BlendMode::Multiply,
            usvg::BlendMode::Screen => BlendMode::Screen,
            usvg::BlendMode::Overlay => BlendMode::Overlay,
            usvg::BlendMode::Darken => BlendMode::Darken,
            usvg::BlendMode::Lighten => BlendMode::Lighten,
            usvg::BlendMode::ColorDodge => BlendMode::ColorDodge,
            usvg::BlendMode::ColorBurn => BlendMode::ColorBurn,
            usvg::BlendMode::HardLight => BlendMode::HardLight,
            usvg::BlendMode::SoftLight => BlendMode::SoftLight,
            usvg::BlendMode::Difference => BlendMode::Difference,
            usvg::BlendMode::Exclusion => BlendMode::Exclusion,
            usvg::BlendMode::Hue => BlendMode::Hue,
            usvg::BlendMode::Saturation => BlendMode::Saturation,
            usvg::BlendMode::Color => BlendMode::Color,
            usvg::BlendMode::Luminosity => BlendMode::Luminosity,
        }
    }
}

// Taken from resvg
/// Calculate the rect of an image after it is scaled using a view box.
#[cfg(feature = "image")]
pub fn image_rect(view_box: &usvg::ViewBox, img_size: Size) -> NonZeroRect {
    let new_size = fit_view_box(img_size, view_box);
    let (x, y) = usvg::utils::aligned_pos(
        view_box.aspect.align,
        view_box.rect.x(),
        view_box.rect.y(),
        view_box.rect.width() - new_size.width(),
        view_box.rect.height() - new_size.height(),
    );

    new_size.to_non_zero_rect(x, y)
}

// Taken from resvg
/// Calculate the new size of a view box that is the result of applying a view box
/// to a certain size.
#[cfg(feature = "image")]
pub fn fit_view_box(size: Size, vb: &usvg::ViewBox) -> usvg::Size {
    let s = vb.rect.size();

    if vb.aspect.align == usvg::Align::None {
        s
    } else if vb.aspect.slice {
        size.expand_to(s)
    } else {
        size.scale_to(s)
    }
}

/// Calculate the scale ratio of a DPI value.
pub fn dpi_ratio(dpi: f32) -> f32 {
    dpi / 72.0
}

/// Compress data using the deflate algorithm.
pub fn deflate(data: &[u8]) -> Vec<u8> {
    const COMPRESSION_LEVEL: u8 = 6;
    miniz_oxide::deflate::compress_to_vec_zlib(data, COMPRESSION_LEVEL)
}

/// Calculate the bbox of a node as a [Rect](usvg::Rect).
pub fn plain_bbox(node: &Node) -> usvg::NonZeroRect {
    calc_node_bbox(node, Transform::default())
        .and_then(|b| b.to_non_zero_rect())
        .unwrap_or(usvg::NonZeroRect::from_xywh(0.0, 0.0, 1.0, 1.0).unwrap())
}

// Taken from resvg
/// Calculate the bbox of a node with a given transform.
fn calc_node_bbox(node: &Node, ts: Transform) -> Option<BBox> {
    match *node.borrow() {
        NodeKind::Path(ref path) => path
            .data
            .bounds()
            .transform(ts)
            .map(|r| {
                let (x, y, w, h) = if let Some(stroke) = &path.stroke {
                    let w = stroke.width.get()
                        / if ts.is_identity() {
                            2.0
                        } else {
                            2.0 / (ts.sx * ts.sy - ts.ky * ts.kx).abs().sqrt()
                        };
                    (r.x() - w, r.y() - w, r.width() + 2.0 * w, r.height() + 2.0 * w)
                } else {
                    (r.x(), r.y(), r.width(), r.height())
                };
                usvg::Rect::from_xywh(x, y, w, h).unwrap_or(r)
            })
            .map(BBox::from),
        NodeKind::Image(ref img) => img.view_box.rect.transform(ts).map(BBox::from),
        NodeKind::Group(_) => {
            let mut bbox = BBox::default();

            for child in node.children() {
                let child_transform = ts.pre_concat(child.transform());
                if let Some(c_bbox) = calc_node_bbox(&child, child_transform) {
                    bbox = bbox.expand(c_bbox);
                }
            }

            // Make sure bbox was changed.
            if bbox.is_default() {
                return None;
            }

            Some(bbox)
        }
        NodeKind::Text(_) => None,
    }
}
