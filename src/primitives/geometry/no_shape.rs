use crate::primitives::{
    Point, Size,
    transform::{CoordinateSpaceTransform, LinearTransform},
};

use super::{Rectangle, Shape, ShapePathIter};

/// A zero-sized type representing the absence of a shape.
///
/// This is useful for `ContentShape` implementations that have no intrinsic shape,
/// avoiding the overhead of `Option<WastedBytesHere>`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NoShape;

impl CoordinateSpaceTransform for NoShape {
    fn applying(&self, _transform: &LinearTransform) -> Self {
        Self
    }

    fn applying_inverse(&self, _transform: &LinearTransform) -> Self {
        Self
    }
}

impl Shape for NoShape {
    type PathElementsIter<'iter> = ShapePathIter<0>;
    #[cfg(feature = "embedded-graphics")]
    type Draw<C: embedded_graphics::prelude::PixelColor> = Self;

    fn path_elements(&self, _tolerance: u16) -> Self::PathElementsIter<'_> {
        ShapePathIter::new([])
    }

    fn bounding_box(&self) -> Rectangle {
        // This is sort of meaningless...
        Rectangle::new(Point::new(0, 0), Size::new(0, 0))
    }
}

#[cfg(feature = "embedded-graphics")]
impl<C: embedded_graphics::prelude::PixelColor> super::embedded_graphics::DrawProvider<NoShape, C> for NoShape {
    fn draw(
        _target: &mut impl embedded_graphics::prelude::DrawTarget<Color = C>,
        _shape: &NoShape,
        _transform: &LinearTransform,
        _style: &embedded_graphics::primitives::PrimitiveStyle<C>,
    ) {
    }
}
