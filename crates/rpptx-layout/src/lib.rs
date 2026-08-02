//! PresentationML inheritance resolution and shape-tree flattening.

mod context;
mod font;
mod style;
mod text;

pub use context::{
    BackgroundContent, BackgroundSource, EffectiveBackground, FlattenedItem, FlattenedSource,
    ResolveCtx,
};
pub use style::{EffectiveShapeStyle, ResolveError};
pub use text::{EffectiveListStyle, EffectiveTextProperties};
