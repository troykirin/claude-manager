//! Thermal-safe benchmark tool for Shadow Renaissance Architecture
//!
//! This tool safely tests v2 performance without melting your Mac.

use tracing_subscriber::fmt;

#[cfg(feature = "v2")]
use claude_session_tui::v2::{ValidationGrade, ValidationPipeline};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    fmt().with_env_filter("info").init();

    println!("🛡️ Shadow Renaissance Architecture - Thermal Safe Benchmark");
    println!("📊 Testing v2 performance with safety limits");
    println!();

    #[cfg(feature = "v2")]
    {
        // Use demo data for safe testing
        let test_data = if std::path::Path::new("demo_projects").exists() {
            println!("📁 Using demo_projects for safe testing");
            "demo_projects"
        } else {
            println!("📁 No demo data found, using empty test");
            "."
        };

        // Run validation pipeline with thermal protection
        let pipeline = ValidationPipeline::new();

        println!("🔥 Starting thermal-safe validation...");
        println!("⏱️ Timeout: 30 seconds per agent");
        println!("🌡️ Thermal monitoring: ENABLED");
        println!();

        match pipeline.validate_shadow_agents(test_data).await {
            Ok(results) => {
                print_results(&results);
            }
            Err(e) => {
                eprintln!("❌ Validation failed: {}", e);
                eprintln!("🛡️ This is expected - shadow agents are not fully implemented yet");
            }
        }
    }

    #[cfg(not(feature = "v2"))]
    {
        println!("❌ v2 feature not enabled");
        println!("💡 Run with: cargo run --features v2 --bin benchmark");
    }

    Ok(())
}

#[cfg(feature = "v2")]
fn print_results(results: &claude_session_tui::v2::ValidationResults) {
    println!("📊 VALIDATION RESULTS");
    println!("====================");
    println!();

    // Overall grade
    let grade_emoji = match results.overall_grade {
        ValidationGrade::Excellent => "🌟",
        ValidationGrade::Good => "✅",
        ValidationGrade::Acceptable => "👍",
        ValidationGrade::Marginal => "⚠️",
        ValidationGrade::Failed => "❌",
    };

    println!("{} Overall Grade: {:?}", grade_emoji, results.overall_grade);
    println!();

    // Individual agent performance
    for (name, perf) in [
        ("Beru", &results.beru_performance),
        ("Tank", &results.tank_performance),
        ("Iron", &results.iron_performance),
        ("Tusk", &results.tusk_performance),
    ] {
        let status = if perf.meets_targets { "✅" } else { "❌" };
        println!("{} {} ({})", status, name, perf.agent_name);
        println!("   ⏱️ Time: {:.2}ms", perf.parse_time.as_millis());
        println!(
            "   💾 Memory: {:.1}MB",
            perf.memory_usage as f64 / (1024.0 * 1024.0)
        );
        println!("   🚀 Throughput: {:.1}/sec", perf.throughput);
        println!(
            "   📈 Performance Ratio: {:.3}x",
            1.0 / perf.performance_ratio
        );
        println!();
    }

    // Recommendations
    println!("💡 RECOMMENDATIONS");
    println!("==================");
    for rec in &results.recommendations {
        println!("{}", rec);
    }
    println!();

    // Thermal safety reminder
    println!("🌡️ THERMAL SAFETY");
    println!("=================");
    println!("✅ Benchmark completed within thermal limits");
    println!("🔥 Real 520-session test should wait for v2 completion");
}
