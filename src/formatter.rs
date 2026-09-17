use colored::*;

use crate::models::{Answer, SystemOneResponse};

pub fn render_progress_bar(prob: f64, width: usize) -> String {
    let clamped = prob.clamp(0.0, 1.0);
    let filled = (clamped * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);

    format!(
        "[{}{}] {:>5.1}%",
        "█".repeat(filled).bright_cyan(),
        "░".repeat(empty).dimmed(),
        clamped * 100.0
    )
}

pub fn render_confidence(conf: f64) -> ColoredString {
    let text = format!("{:.1}%", conf * 100.0);
    if conf >= 0.80 {
        format!("● High ({})", text).bright_green().bold()
    } else if conf >= 0.50 {
        format!("▲ Moderate ({})", text).bright_yellow().bold()
    } else {
        format!("■ Low ({})", text).bright_red().bold()
    }
}

pub fn print_pretty_summary(res: &SystemOneResponse, elapsed_ms: u128) {
    println!();
    println!(
        "{} {} {} {}",
        "───".dimmed(),
        "TypeSafe System One Judgment".bold().bright_white(),
        format!("({})", res.model).dimmed(),
        "───".dimmed()
    );

    let mut keys: Vec<&String> = res.answers.keys().collect();
    keys.sort();

    for key in keys {
        let answer = &res.answers[key];
        println!();
        println!("{} {}", "▶".bright_blue(), key.bold());

        match answer {
            Answer::Choice(choice) => {
                println!(
                    "  {} {}   {}",
                    "Selected:".dimmed(),
                    choice.choice.bright_green().bold(),
                    render_confidence(choice.confidence)
                );
                println!("  {}", "Probabilities:".dimmed());

                let mut sorted_probs: Vec<(&String, &f64)> = choice.probabilities.iter().collect();
                sorted_probs.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

                for (opt, prob) in sorted_probs {
                    let is_winner = opt == &choice.choice;
                    let prefix = if is_winner { "  ✓".bright_green() } else { "   ".normal() };
                    println!(
                        "{} {:<20} {}",
                        prefix,
                        if is_winner { opt.bold() } else { opt.normal() },
                        render_progress_bar(*prob, 16)
                    );
                }
            }
            Answer::Noul(noul) => {
                let p = noul.noul;
                let verdict = if p >= 0.70 {
                    "YES".bright_green().bold()
                } else if p >= 0.50 {
                    "LEANING YES".bright_yellow().bold()
                } else if p >= 0.30 {
                    "LEANING NO".bright_yellow().bold()
                } else {
                    "NO".bright_red().bold()
                };

                println!("  {} {}   {}", "Verdict:".dimmed(), verdict, render_progress_bar(p, 20));
            }
            Answer::Score(score) => {
                println!(
                    "  {} {}   {}",
                    "Score:".dimmed(),
                    format!("{:.2}", score.score).bright_magenta().bold(),
                    render_confidence(score.confidence)
                );

                if !score.probabilities.is_empty() {
                    println!("  {}", "Level Probabilities:".dimmed());
                    let mut sorted_levels: Vec<(&String, &f64)> = score.probabilities.iter().collect();
                    sorted_levels.sort_by_key(|(k, _)| k.parse::<u64>().unwrap_or(0));

                    for (lvl, prob) in sorted_levels {
                        let desc = score
                            .legend
                            .get(lvl)
                            .map(|s| s.as_str())
                            .unwrap_or("");
                        println!(
                            "    [{}] {:<28} {}",
                            lvl.dimmed(),
                            desc.dimmed(),
                            render_progress_bar(*prob, 14)
                        );
                    }
                }
            }
        }
    }

    println!();
    println!(
        "{} {} ms | {} in_tokens, {} out_tokens",
        "⚡".bright_yellow(),
        elapsed_ms.to_string().bold(),
        res.usage.input_tokens.to_string().dimmed(),
        res.usage.output_tokens.to_string().dimmed()
    );
    println!();
}
