/// `HandleSmallArea` is a trait that represents a component that can handle small area events.
/// Implementors of this trait can be notified when the area they are rendering in is too small to be useful.
/// This can be useful for components that require a minimum amount of space to be useful.
pub trait HandleSmallArea {
  /// Handle a small area event.
  ///
  /// # Arguments
  ///
  /// * `small` - A boolean indicating if the area is too small.
  ///
  #[allow(unused_variables)]
  fn small_area(&mut self, small: bool) {}
}
