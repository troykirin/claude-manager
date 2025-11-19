// Quick test for path decoding logic
fn decode_project_path(encoded_path: &str) -> String {
    let home_dir = "/Users/tryk";
    
    // Split the encoded path by "-" to get segments
    let segments: Vec<&str> = encoded_path.split('-').collect();

    // Filter out empty segments and reconstruct path
    let path_segments: Vec<&str> = segments
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect();

    if path_segments.is_empty() {
        return "unknown".to_string();
    }

    // Check if this is an absolute path (starts with Users, home, var, etc.)
    let full_path = if path_segments.get(0) == Some(&"Users")
        || path_segments.get(0) == Some(&"home")
        || path_segments.get(0) == Some(&"var") {
        // This looks like an absolute path
        format!("/{}", path_segments.join("/"))
    } else {
        // Relative path
        path_segments.join("/")
    };

    // Convert absolute home paths to ~/
    if let Some(home) = home_dir.strip_suffix("/") {
        if full_path.starts_with(home) && home.len() > 1 {
            let remainder = &full_path[home.len()..];
            return format!("~{}", remainder);
        }
    }
    if full_path.starts_with(home_dir) && home_dir.len() > 1 {
        let remainder = &full_path[home_dir.len()..];
        return format!("~{}", remainder);
    }

    full_path
}

fn main() {
    // Test cases
    let test_cases = vec![
        ("-Users-tryk--nabia", "~/nabia"),
        ("-Users-tryk--nabia-tools-project", "~/nabia/tools/project"),
        ("-Users-tryk--nabi--tools--claude-manager", "~/nabi/tools/claude-manager"),
        ("unknown-path", "unknown/path"),
    ];

    for (encoded, expected) in test_cases {
        let result = decode_project_path(encoded);
        let status = if result == expected { "✓" } else { "✗" };
        println!("{} {} -> {} (expected: {})", status, encoded, result, expected);
    }
}
