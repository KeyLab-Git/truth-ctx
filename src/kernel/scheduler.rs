pub fn generate_final_prompt(user_input: &str, active_truth: &str) -> String {
    format!(
        "{}\n\n[KERNEL_CRITICAL_STATE]\n{}\n[END_STATE]",
        user_input, active_truth
    )
}