mod handle_animation;
mod handle_photo;
mod handle_unknown;
mod handle_video;

pub use handle_animation::handle_animation;
pub use handle_photo::handle_photo;
pub use handle_unknown::handle_unknown;
pub use handle_video::handle_video;
