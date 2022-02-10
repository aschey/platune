use itertools::Itertools;

pub(crate) fn generate_parameterized_bindings(start: usize, count: usize) -> String {
    (start..start + count)
        .map(|i| "$".to_owned() + &i.to_string())
        .collect_vec()
        .join(",")
}
