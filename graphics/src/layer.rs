//! Organize rendering primitives into a flattened list of layers.
use crate::svg;
use crate::triangle;
use crate::{image, Color};
use crate::{
    Background, Font, Gradient, HorizontalAlignment, Point, Primitive,
    Rectangle, Size, Vector, VerticalAlignment, Viewport,
};
use iced_native::LinearGradient;

/// A group of primitives that should be clipped together.
#[derive(Debug, Clone)]
pub struct Layer<'a> {
    /// The clipping bounds of the [`Layer`].
    pub bounds: Rectangle,

    /// The quads of the [`Layer`].
    pub quads: Vec<Quad>,

    /// The gradient quads of the [`Layer`].
    pub gradient_quads: Vec<GradientQuad>,

    /// The triangle meshes of the [`Layer`].
    pub meshes: Vec<Mesh<'a>>,

    /// The text of the [`Layer`].
    pub text: Vec<Text<'a>>,

    /// The images of the [`Layer`].
    pub images: Vec<Image>,
}

impl<'a> Layer<'a> {
    /// Creates a new [`Layer`] with the given clipping bounds.
    pub fn new(bounds: Rectangle) -> Self {
        Self {
            bounds,
            quads: Vec::new(),
            gradient_quads: Vec::new(),
            meshes: Vec::new(),
            text: Vec::new(),
            images: Vec::new(),
        }
    }

    /// Creates a new [`Layer`] for the provided overlay text.
    ///
    /// This can be useful for displaying debug information.
    pub fn overlay(lines: &'a [impl AsRef<str>], viewport: &Viewport) -> Self {
        let mut overlay =
            Layer::new(Rectangle::with_size(viewport.logical_size()));

        for (i, line) in lines.iter().enumerate() {
            let text = Text {
                content: line.as_ref(),
                bounds: Rectangle::new(
                    Point::new(11.0, 11.0 + 25.0 * i as f32),
                    Size::INFINITY,
                ),
                color: [0.9, 0.9, 0.9, 1.0],
                size: 20.0,
                font: Font::Default,
                horizontal_alignment: HorizontalAlignment::Left,
                vertical_alignment: VerticalAlignment::Top,
            };

            overlay.text.push(text);

            overlay.text.push(Text {
                bounds: text.bounds + Vector::new(-1.0, -1.0),
                color: [0.0, 0.0, 0.0, 1.0],
                ..text
            });
        }

        overlay
    }

    /// Distributes the given [`Primitive`] and generates a list of layers based
    /// on its contents.
    pub fn generate(
        primitive: &'a Primitive,
        viewport: &Viewport,
    ) -> Vec<Self> {
        let first_layer =
            Layer::new(Rectangle::with_size(viewport.logical_size()));

        let mut layers = vec![first_layer];

        Self::process_primitive(
            &mut layers,
            Vector::new(0.0, 0.0),
            primitive,
            0,
        );

        layers
    }

    fn process_primitive(
        layers: &mut Vec<Self>,
        translation: Vector,
        primitive: &'a Primitive,
        current_layer: usize,
    ) {
        match primitive {
            Primitive::None => {}
            Primitive::Group { primitives } => {
                // TODO: Inspect a bit and regroup (?)
                for primitive in primitives {
                    Self::process_primitive(
                        layers,
                        translation,
                        primitive,
                        current_layer,
                    )
                }
            }
            Primitive::Text {
                content,
                bounds,
                size,
                color,
                font,
                horizontal_alignment,
                vertical_alignment,
            } => {
                let layer = &mut layers[current_layer];

                layer.text.push(Text {
                    content,
                    bounds: *bounds + translation,
                    size: *size,
                    color: color.into_linear(),
                    font: *font,
                    horizontal_alignment: *horizontal_alignment,
                    vertical_alignment: *vertical_alignment,
                });
            }
            Primitive::Quad {
                bounds,
                background,
                border_radius,
                border_width,
                border_color,
            } => {
                let layer = &mut layers[current_layer];

                // TODO: Move some of these computations to the GPU (?)
                match background {
                    Background::Color(color) => {
                        layer.quads.push(Quad {
                            position: [
                                bounds.x + translation.x,
                                bounds.y + translation.y,
                            ],
                            size: [bounds.width, bounds.height],
                            color: color.into_linear(),
                            border_radius: *border_radius,
                            border_width: *border_width,
                            border_color: border_color.into_linear(),
                        });
                    }
                    Background::Gradient(gradient) => match gradient {
                        Gradient::LinearGradient(gradient) => {
                            if gradient.stops.is_empty() {
                                return;
                            }

                            let direction = gradient.direction;
                            let border_radius = *border_radius;

                            let mut stop_pairs = vec![];
                            let mut last_stop = gradient.stops[0];
                            for stop in gradient.stops.iter().skip(1) {
                                stop_pairs.push((last_stop, stop));
                                last_stop = *stop;
                            }

                            for (index, (start_stop, end_stop)) in
                                stop_pairs.iter().enumerate()
                            {
                                let border_radius = if index == 0
                                    || index == gradient.stops.len() - 1
                                {
                                    border_radius
                                } else {
                                    0.0
                                };
                                let range =
                                    end_stop.percentage - start_stop.percentage;
                                let offset =
                                    start_stop.percentage * bounds.width;
                                let position = [
                                    bounds.x + translation.x + offset,
                                    bounds.y + translation.y,
                                ];

                                layer.gradient_quads.push(GradientQuad {
                                    position,
                                    size: [bounds.width * range, bounds.height],
                                    start_color: start_stop.color.into_linear(),
                                    end_color: end_stop.color.into_linear(),
                                    direction: 0.0,
                                    start_percentage: start_stop.percentage,
                                    stop_percentage: end_stop.percentage,
                                    border_radius: 0.0,
                                })
                            }
                        }
                    },
                }
            }
            Primitive::Mesh2D { buffers, size } => {
                let layer = &mut layers[current_layer];

                let bounds = Rectangle::new(
                    Point::new(translation.x, translation.y),
                    *size,
                );

                // Only draw visible content
                if let Some(clip_bounds) = layer.bounds.intersection(&bounds) {
                    layer.meshes.push(Mesh {
                        origin: Point::new(translation.x, translation.y),
                        buffers,
                        clip_bounds,
                    });
                }
            }
            Primitive::Clip {
                bounds,
                offset,
                content,
            } => {
                let layer = &mut layers[current_layer];
                let translated_bounds = *bounds + translation;

                // Only draw visible content
                if let Some(clip_bounds) =
                    layer.bounds.intersection(&translated_bounds)
                {
                    let clip_layer = Layer::new(clip_bounds);
                    layers.push(clip_layer);

                    Self::process_primitive(
                        layers,
                        translation
                            - Vector::new(offset.x as f32, offset.y as f32),
                        content,
                        layers.len() - 1,
                    );
                }
            }
            Primitive::Translate {
                translation: new_translation,
                content,
            } => {
                Self::process_primitive(
                    layers,
                    translation + *new_translation,
                    &content,
                    current_layer,
                );
            }
            Primitive::Cached { cache } => {
                Self::process_primitive(
                    layers,
                    translation,
                    &cache,
                    current_layer,
                );
            }
            Primitive::Image { handle, bounds } => {
                let layer = &mut layers[current_layer];

                layer.images.push(Image::Raster {
                    handle: handle.clone(),
                    bounds: *bounds + translation,
                });
            }
            Primitive::Svg { handle, bounds } => {
                let layer = &mut layers[current_layer];

                layer.images.push(Image::Vector {
                    handle: handle.clone(),
                    bounds: *bounds + translation,
                });
            }
        }
    }
}

/// A colored rectangle with a border.
///
/// This type can be directly uploaded to GPU memory.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Quad {
    /// The position of the [`Quad`].
    pub position: [f32; 2],

    /// The size of the [`Quad`].
    pub size: [f32; 2],

    /// The color of the [`Quad`], in __linear RGB__.
    pub color: [f32; 4],

    /// The border color of the [`Quad`], in __linear RGB__.
    pub border_color: [f32; 4],

    /// The border radius of the [`Quad`].
    pub border_radius: f32,

    /// The border width of the [`Quad`].
    pub border_width: f32,
}

/// A gradient colored rectangle with a border.
///
/// This type can be directly uploaded to GPU memory.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GradientQuad {
    /// The position of the [`GradientQuad`].
    pub position: [f32; 2],

    /// The size of the [`GradientQuad`].
    pub size: [f32; 2],

    /// The color of the [`GradientQuad`], in __linear RGB__.
    pub start_color: [f32; 4],

    /// The color of the [`GradientQuad`], in __linear RGB__.
    pub end_color: [f32; 4],

    /// The direction of the [`GradientQuad`] fill in radians
    pub direction: f32,

    /// Where this gradient starts within size
    pub start_percentage: f32,

    /// Where this gradient ends within size
    pub stop_percentage: f32,

    /// The border radius of the [`GradientQuad`].
    pub border_radius: f32,
}

/// A mesh of triangles.
#[derive(Debug, Clone, Copy)]
pub struct Mesh<'a> {
    /// The origin of the vertices of the [`Mesh`].
    pub origin: Point,

    /// The vertex and index buffers of the [`Mesh`].
    pub buffers: &'a triangle::Mesh2D,

    /// The clipping bounds of the [`Mesh`].
    pub clip_bounds: Rectangle<f32>,
}

/// A paragraph of text.
#[derive(Debug, Clone, Copy)]
pub struct Text<'a> {
    /// The content of the [`Text`].
    pub content: &'a str,

    /// The layout bounds of the [`Text`].
    pub bounds: Rectangle,

    /// The color of the [`Text`], in __linear RGB_.
    pub color: [f32; 4],

    /// The size of the [`Text`].
    pub size: f32,

    /// The font of the [`Text`].
    pub font: Font,

    /// The horizontal alignment of the [`Text`].
    pub horizontal_alignment: HorizontalAlignment,

    /// The vertical alignment of the [`Text`].
    pub vertical_alignment: VerticalAlignment,
}

/// A raster or vector image.
#[derive(Debug, Clone)]
pub enum Image {
    /// A raster image.
    Raster {
        /// The handle of a raster image.
        handle: image::Handle,

        /// The bounds of the image.
        bounds: Rectangle,
    },
    /// A vector image.
    Vector {
        /// The handle of a vector image.
        handle: svg::Handle,

        /// The bounds of the image.
        bounds: Rectangle,
    },
}

#[allow(unsafe_code)]
unsafe impl bytemuck::Zeroable for Quad {}

#[allow(unsafe_code)]
unsafe impl bytemuck::Pod for Quad {}

#[allow(unsafe_code)]
unsafe impl bytemuck::Zeroable for GradientQuad {}

#[allow(unsafe_code)]
unsafe impl bytemuck::Pod for GradientQuad {}
