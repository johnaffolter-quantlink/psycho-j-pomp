//! Forge-led agents on real Git: a film of measured results, not a simulation.
//!
//! Every number on screen is read from the scenario suite's `results.json`
//! (cloudflare-agent-forge `packages/forge/scripts/scenarios.mjs`): agents start
//! and are coordinated through the Forge SDK, attempts run in real worktrees,
//! and `git merge-tree` plus the combined tests are the ground truth. No
//! narration, no paid calls.
//!
//!   cargo run -p psychopomp-forge-scenarios -- <results.json> [out.reel.json]
//!   psychopomp plan render <out.reel.json> output/forge-scenarios.mp4 --theme neutral
use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use psychopomp::{
    author::{PlanBuilder, SECOND, seconds},
    axis::AxisPlan,
    bars::{BarSeriesPlan, BarsActor, BarsPlan},
    caption::{CaptionActor, CaptionPlan, CaptionSpanPlan},
    checklist::{ChecklistActor, ChecklistItemPlan, ChecklistPlan, Outcome},
    chrome::{chip, footer, header},
    plan::{ReelPlan, ReelSegmentPlan, ReelTransitionStyle, ScenePlan},
    readout::ReadoutFormat,
    stage::{StageActor, StageElement, StagePlan},
    tone::Tone,
};
use serde_json::Value;

fn span(text: &str, tone: Tone) -> CaptionSpanPlan {
    CaptionSpanPlan::new(text, tone)
}

fn line(
    scene: &mut PlanBuilder,
    id: &str,
    y: f32,
    size: f32,
    spans: Vec<CaptionSpanPlan>,
    at: u64,
) -> Result<()> {
    let mut caption =
        CaptionActor::declare(scene, id, &CaptionPlan::line([140.0, y], size, spans))?;
    caption.type_in(scene, at, 60.0, 0.6);
    Ok(())
}

/// A round axis step (1, 2 or 5 times a power of ten) giving about four ticks.
fn nice_step(max: f64) -> f32 {
    let raw = (max / 4.0).max(1.0);
    let power = 10f64.powf(raw.log10().floor());
    let step = [1.0, 2.0, 5.0, 10.0]
        .into_iter()
        .map(|m| m * power)
        .find(|s| *s >= raw)
        .unwrap_or(10.0 * power);
    step as f32
}

fn num(v: &Value) -> f64 {
    v.as_f64().unwrap_or(0.0)
}

fn title(r: &Value, duration: u64) -> Result<(ScenePlan, u64)> {
    let mut s = PlanBuilder::new("forge-title", duration);
    let mut h = header(
        &mut s,
        "FORGE",
        "agents start, coordinate and are judged on real Git",
    )?;
    h.type_in(&mut s, seconds(0.2), 48.0, 0.7);
    chip(&mut s, "chip", Tone::Success, "measured")?.show(&mut s, seconds(0.5));
    let runs = r["scenarios"].as_array().map_or(0, Vec::len);
    let kinds = r["byKind"].as_array().map_or(0, Vec::len);
    let starts: u64 = r["scenarios"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|x| x["agents"].as_u64().unwrap_or(0))
        .sum();
    line(
        &mut s,
        "runs",
        420.0,
        50.0,
        vec![
            span(&format!("{runs} runs"), Tone::Accent),
            span(
                &format!("  ·  {kinds} scenario kinds  ·  {starts} agent starts"),
                Tone::Plain,
            ),
        ],
        seconds(1.0),
    )?;
    line(
        &mut s,
        "entry",
        520.0,
        36.0,
        vec![
            span("every agent enters through ", Tone::Plain),
            span("forge(plan).using(runtime).run(id)", Tone::Accent),
        ],
        seconds(2.4),
    )?;
    line(
        &mut s,
        "truth",
        590.0,
        36.0,
        vec![
            span("truth is ", Tone::Plain),
            span("git merge-tree", Tone::Accent),
            span(" and the combined tree's own tests", Tone::Plain),
        ],
        seconds(3.6),
    )?;
    let end = seconds(6.5);
    s.cue("title", 0, duration);
    Ok((s.finish().context("title")?, end))
}

fn pipeline(_: &Value, duration: u64) -> Result<(ScenePlan, u64)> {
    let mut s = PlanBuilder::new("forge-pipeline", duration);
    header(
        &mut s,
        "FLOW",
        "one entry point, graph-gated, receipts for everything",
    )?
    .show(&mut s, 0);
    let names = [
        ("plan", "declared plan"),
        ("graph", "graph lookahead"),
        ("astar", "A* schedule"),
        ("worktrees", "isolated worktrees"),
        ("judge", "git judge"),
    ];
    let mut elements = vec![];
    for (i, (id, label)) in names.iter().enumerate() {
        elements.push(StageElement::card(
            id,
            [260.0 + i as f32 * 350.0, 520.0, 0.0],
            [300.0, 100.0],
            label,
        ));
    }
    for i in 0..names.len() - 1 {
        let beam = format!("b{i}");
        elements.push(StageElement::beam(&beam, names[i].0, names[i + 1].0));
        elements.push(StageElement::packet(&format!("p{i}"), &beam));
    }
    let mut stage = StageActor::declare(
        &mut s,
        "stage",
        &StagePlan {
            post: Default::default(),
            elements,
        },
    )?;
    let mut t = seconds(0.4);
    for (i, (id, _)) in names.iter().enumerate() {
        t = stage.settle_in(&mut s, id, t);
        if i + 1 < names.len() {
            let wired = stage.connect(&mut s, &format!("b{i}"), t, 0.35);
            t = stage.send(&mut s, &format!("p{i}"), wired, 0.45);
        }
    }
    stage.land(&mut s, "judge", t);
    let mut f = footer(
        &mut s,
        "footer",
        vec![
            span("same operation id, same receipt: ", Tone::Plain),
            span("replay executes nothing", Tone::Accent),
        ],
    )?;
    let footer_at = t + seconds(0.3);
    f.type_in(&mut s, footer_at, 46.0, 0.6);
    let end = footer_at + seconds(1.5);
    s.cue("pipeline", 0, duration);
    Ok((s.finish().context("pipeline")?, end))
}

fn ablations(r: &Value, duration: u64) -> Result<(ScenePlan, u64)> {
    let mut s = PlanBuilder::new("forge-ablations", duration);
    header(
        &mut s,
        "ABLATIONS",
        "what each planner batches side by side, judged by Git",
    )?
    .show(&mut s, 0);
    let rows = r["ablations"].as_array().context("ablations")?;
    let max_conflicts = rows
        .iter()
        .map(|a| num(&a["concurrentConflicts"]))
        .fold(1.0, f64::max);
    let max_cost = rows.iter().map(|a| num(&a["cost"])).fold(1.0, f64::max);
    let plan = |id: &str, origin: [f32; 2], max: f64, label: &str, unit: &str| {
        rows.iter().fold(
            BarsPlan::new(
                origin,
                620.0,
                AxisPlan::new([0.0, (max * 1.15) as f32]).every(nice_step(max)),
            )
            .row_height(84.0)
            .size(26.0)
            .series(BarSeriesPlan::new(
                id,
                label,
                if id == "conflicts" {
                    Tone::Warning
                } else {
                    Tone::Accent
                },
            ))
            .readout(ReadoutFormat::new(0).grouped().unit(unit)),
            |p, a| {
                p.row(
                    a["ablation"]
                        .as_str()
                        .unwrap_or("?")
                        .replace('-', "_")
                        .as_str(),
                    a["ablation"].as_str().unwrap_or("?"),
                )
            },
        )
    };
    let mut conflicts = BarsActor::declare(
        &mut s,
        "conflicts",
        &plan(
            "conflicts",
            [300.0, 300.0],
            max_conflicts,
            "real conflicts batched together",
            "",
        ),
    )?;
    let mut cost = BarsActor::declare(
        &mut s,
        "cost",
        &plan(
            "cost",
            [1160.0, 300.0],
            max_cost,
            "schedule cost (plan units)",
            "",
        ),
    )?;
    conflicts.show(&mut s, seconds(0.3));
    cost.show(&mut s, seconds(0.3));
    let values = |key: &str| {
        rows.iter()
            .map(|a| {
                (
                    a["ablation"].as_str().unwrap_or("?").replace('-', "_"),
                    num(&a[key]) as f32,
                )
            })
            .collect::<Vec<_>>()
    };
    let c = values("concurrentConflicts");
    let k = values("cost");
    conflicts.grow(
        &mut s,
        seconds(1.0),
        "conflicts",
        &c.iter()
            .map(|(id, v)| (id.as_str(), *v))
            .collect::<Vec<_>>(),
    )?;
    cost.grow(
        &mut s,
        seconds(3.0),
        "cost",
        &k.iter()
            .map(|(id, v)| (id.as_str(), *v))
            .collect::<Vec<_>>(),
    )?;
    let full = rows
        .iter()
        .find(|a| a["ablation"] == "full")
        .context("full")?;
    let no_graph = rows
        .iter()
        .find(|a| a["ablation"] == "no-graph")
        .context("no-graph")?;
    let premium = 100.0 * (num(&full["cost"]) / num(&no_graph["cost"]).max(1.0) - 1.0);
    let mut f = footer(
        &mut s,
        "footer",
        vec![
            span(
                &format!(
                    "without the graph: {} real conflicts batched together",
                    num(&no_graph["concurrentConflicts"])
                ),
                Tone::Warning,
            ),
            span(
                &format!(
                    "  ·  full: {}  ·  for {premium:+.0}% schedule cost",
                    num(&full["concurrentConflicts"])
                ),
                Tone::Plain,
            ),
        ],
    )?;
    f.type_in(&mut s, seconds(5.5), 46.0, 0.7);
    let end = seconds(9.5);
    s.cue("ablations", 0, duration);
    Ok((s.finish().context("ablations")?, end))
}

fn predictors(r: &Value, duration: u64) -> Result<(ScenePlan, u64)> {
    let mut s = PlanBuilder::new("forge-predictors", duration);
    header(
        &mut s,
        "PREDICTION",
        "warned before any code existed, scored against Git",
    )?
    .show(&mut s, 0);
    let rows = r["predictors"].as_array().context("predictors")?;
    let plan = rows.iter().fold(
        BarsPlan::new(
            [520.0, 330.0],
            900.0,
            AxisPlan::new([0.0, 100.0]).every(25.0).label("percent"),
        )
        .row_height(120.0)
        .size(28.0)
        .series(BarSeriesPlan::new("precision", "precision", Tone::Muted))
        .series(BarSeriesPlan::new("recall", "recall", Tone::Accent))
        .readout(ReadoutFormat::new(0).unit("%")),
        |p, x| {
            let id = x["predictor"].as_str().unwrap_or("?");
            p.row(
                id,
                if id == "graph" {
                    "graph exposure"
                } else {
                    "file overlap"
                },
            )
        },
    );
    let mut bars = BarsActor::declare(&mut s, "predictors", &plan)?;
    bars.show(&mut s, seconds(0.3));
    let pct = |key: &str| {
        rows.iter()
            .map(|x| {
                (
                    x["predictor"].as_str().unwrap_or("?").to_owned(),
                    (num(&x[key]) * 100.0) as f32,
                )
            })
            .collect::<Vec<_>>()
    };
    let p = pct("precision");
    let rc = pct("recall");
    bars.grow(
        &mut s,
        seconds(1.0),
        "precision",
        &p.iter()
            .map(|(id, v)| (id.as_str(), *v))
            .collect::<Vec<_>>(),
    )?;
    bars.grow(
        &mut s,
        seconds(2.6),
        "recall",
        &rc.iter()
            .map(|(id, v)| (id.as_str(), *v))
            .collect::<Vec<_>>(),
    )?;
    let graph = rows
        .iter()
        .find(|x| x["predictor"] == "graph")
        .context("graph")?;
    let mut f = footer(
        &mut s,
        "footer",
        vec![
            span(
                &format!("graph exposure missed {} real conflicts", num(&graph["fn"])),
                Tone::Accent,
            ),
            span(
                "  ·  false alarms are serialized, never shipped together",
                Tone::Plain,
            ),
        ],
    )?;
    f.type_in(&mut s, seconds(4.5), 46.0, 0.7);
    let end = seconds(8.5);
    s.cue("predictors", 0, duration);
    Ok((s.finish().context("predictors")?, end))
}

fn faults(r: &Value, duration: u64) -> Result<(ScenePlan, u64)> {
    let mut s = PlanBuilder::new("forge-faults", duration);
    header(
        &mut s,
        "FAULTS",
        "every failure ends in a known, recoverable state",
    )?
    .show(&mut s, 0);
    let rows = r["faults"].as_array().context("faults")?;
    let plan = rows.iter().fold(
        ChecklistPlan::new([300.0, 230.0], 1320.0)
            .title("runtime faults through the same SDK")
            .size(28.0)
            .panel(),
        |p, x| {
            let id = x["fault"].as_str().unwrap_or("?").replace('-', "_");
            let result = match x["fault"].as_str().unwrap_or("") {
                "reused-id-new-plan" => "reuse refused".to_owned(),
                "composed-dependency" => "producer composed".to_owned(),
                _ => x["status"].as_str().unwrap_or("?").to_owned(),
            };
            p.item(
                ChecklistItemPlan::new(id, x["fault"].as_str().unwrap_or("?").replace('-', " "))
                    .result(result)
                    .failure("unexpected"),
            )
        },
    );
    let mut list = ChecklistActor::declare(&mut s, "faults", &plan)?;
    list.show(&mut s, seconds(0.2));
    let mut t = list.reveal(&mut s, seconds(0.4))?;
    for x in rows {
        let id = x["fault"].as_str().unwrap_or("?").replace('-', "_");
        list.start(&mut s, &id, t)?;
        t = list.resolve(
            &mut s,
            &id,
            t + seconds(0.35),
            if x["ok"].as_bool() == Some(true) {
                Outcome::Done
            } else {
                Outcome::Failed
            },
        )?;
    }
    let passed = r["acceptance"].as_array().map_or(0, |a| {
        a.iter()
            .filter(|x| x["pass"].as_bool() == Some(true))
            .count()
    });
    let total = r["acceptance"].as_array().map_or(0, Vec::len);
    let mut f = footer(
        &mut s,
        "footer",
        vec![
            span(&format!("acceptance {passed}/{total}"), Tone::Success),
            span(
                "  ·  Forge never moved trunk; nothing landed or pushed",
                Tone::Plain,
            ),
        ],
    )?;
    let footer_at = t + seconds(0.3);
    f.type_in(&mut s, footer_at, 46.0, 0.6);
    let end = footer_at + seconds(2.5);
    s.cue("faults", 0, duration);
    Ok((s.finish().context("faults")?, end))
}

/// Every segment is as long as what it shows: a dry run with room to spare
/// measures where its last move ends, then the real plan is built to fit.
fn fit(r: &Value, build: fn(&Value, u64) -> Result<(ScenePlan, u64)>) -> Result<ScenePlan> {
    let (_, end) = build(r, 120 * SECOND)?;
    Ok(build(r, end + seconds(1.0))?.0)
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let input = PathBuf::from(
        args.get(1)
            .context("usage: <results.json> [out.reel.json]")?,
    );
    let output = args
        .get(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/forge-scenarios.reel.json"));
    let results: Value = serde_json::from_str(&fs::read_to_string(&input)?)?;
    let segment = |plan: ScenePlan, first: bool| ReelSegmentPlan {
        transition_nanos: if first { 0 } else { seconds(0.5) },
        transition_style: ReelTransitionStyle::Dip,
        transition_focus: None,
        transition_wipe: None,
        plan,
    };
    let reel = ReelPlan {
        version: ReelPlan::VERSION,
        id: "forge-scenarios".to_owned(),
        segments: vec![
            segment(fit(&results, title)?, true),
            segment(fit(&results, pipeline)?, false),
            segment(fit(&results, ablations)?, false),
            segment(fit(&results, predictors)?, false),
            segment(fit(&results, faults)?, false),
        ],
    };
    reel.validate()?;
    if let Some(dir) = output.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(&output, serde_json::to_string_pretty(&reel)? + "\n")?;
    eprintln!(
        "wrote {} ({:.1}s)",
        output.display(),
        reel.duration_nanos() as f64 / 1e9
    );
    Ok(())
}
