mod normalize;
mod parser;
mod renderer;
mod tests;

#[allow(unused_imports)]
pub use normalize::normalize_markdown;

pub use parser::markdown_to_markup;
pub use renderer::markup_to_markdown;
