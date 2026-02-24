/// Generic selector helper.
pub fn select<T>(true_case: T, false_case: T, condition: bool) -> T {
  if condition { true_case } else { false_case }
}
