// Updated test for path decoding logic with correct "--" separator
fn decode_project_path(encoded_path: &str) -> String {
    let home_dir = "/Users/tryk";
    
    // Claude encodes paths with "--" as the path separator
    // Strip leading "-" and then split by "--"
    let trimmed = encoded_path.strip_prefix('-').unwrap_or(encoded_path);
    let path_segments: Vec<&str> = trimmed.split("--").collect();

    if path_segments.is_empty() {
        return "unknown".to_string();
    }

    // Reconstruct as absolute path
    let full_path = format!("/{}", path_segments.join("/"));

    // Convert absolute home paths to ~/
    // Try with trailing slash stripped first
    if let Some(home) = home_dir.strip_suffix("/") {
        if full_path.starts_with(home) && home.len() > 1 {
            let remainder = &full_path[home.len()..];
            return format!("~{}", remainder);
        }
    }

    // Then try with the home dir as-is
    if full_path.starts_with(home_dir) && home_dir.len() > 1 {
        let remainder = &full_path[home_dir.len()..];
        return format!("~{}", remainder);
    }

    // If not under home dir, return the full path
    full_path
}

fn main() {
    // Test cases based on actual Claude project paths
    let test_cases = vec![
        ("-Users-tryk", "~"),
        ("-Users-tryk--nabia", "~/nabia"),
        ("-Users-tryk--config-nabi", "~/.config/nabi"),
        ("-Users-tryk--config-nabi-cli", "~/.config/nabi-cli"),
        ("-Users-tryk--claude", "~/.claude"),
        ("-Users-tryk--cargo", "~/.cargo"),
        ("-Users-tryk--nabi", "~/nabi"),
    ];

    println!("Testing path decoding:");
    for (encoded, expected) in test_cases {
        let result = decode_project_path(encoded);
        let status = if result == expected { "✓" } else { "✗" };
        println!("{} {:40} -> {:30} (expected: {})", status, encoded, result, expected);
    }
}
