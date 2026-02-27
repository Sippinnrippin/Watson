pub fn generate_variations(username: &str) -> Vec<String> {
    let mut variations = vec![username.to_string()];

    // Add underscore prefix/suffix
    variations.push(format!("_{}", username));
    variations.push(format!("{}_", username));

    // Add dots
    variations.push(username.replace("_", "."));
    variations.push(username.replace("-", "."));

    // Add underscore
    variations.push(username.replace(".", "_"));
    variations.push(username.replace("-", "_"));

    // Add dash prefix/suffix
    variations.push(format!("-{}", username));
    variations.push(format!("{}-", username));

    // Common suffixes
    variations.push(format!("{}1", username));
    variations.push(format!("{}12", username));
    variations.push(format!("{}123", username));
    variations.push(format!("{}0", username));

    // Common prefixes
    variations.push(format!("the_{}", username));
    variations.push(format!("_{}", username));
    variations.push(format!("real_{}", username));

    // Case variations
    variations.push(username.to_lowercase());
    variations.push(username.to_uppercase());

    // Remove duplicates
    variations.sort();
    variations.dedup();

    variations
}
