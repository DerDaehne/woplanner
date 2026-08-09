//! Austauschformat für Trainingspläne — siehe Notiz `concept-workout-yaml`.
//!
//! Bewusst ohne YAML-Bibliothek: crates.io ist in dieser Umgebung nicht
//! erreichbar (siehe Ticket #715). Gelesen wird deshalb nur die Teilmenge, die
//! das Format braucht — Block-Stil, zwei Ebenen, Skalare und ein Blockstring.
//! Alles andere wird als Fehler gemeldet, nicht geraten.

use std::fmt;

/// Ein Plan, wie er in der Datei steht: ohne IDs, Zeitstempel und Nutzerbezug.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkoutPlan {
    pub name: String,
    pub description: Option<String>,
    pub schedule_type: String,
    pub schedule_day: Option<i64>,
    pub exercises: Vec<PlanExercise>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanExercise {
    pub name: String,
    pub instructions: Option<String>,
    pub video_url: Option<String>,
    pub sets: i64,
    pub weight: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line > 0 {
            write!(f, "line {}: {}", self.line, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

fn err<T>(line: usize, message: impl Into<String>) -> Result<T, ParseError> {
    Err(ParseError { line, message: message.into() })
}

pub const FORMAT_VERSION: i64 = 1;
pub const MAX_INPUT_BYTES: usize = 64 * 1024;
pub const MAX_EXERCISES: usize = 50;
pub const MAX_SETS: i64 = 50;
pub const MAX_NAME_LEN: usize = 100;

const SCHEDULE_TYPES: [&str; 4] = ["manual", "rotation", "weekly", "disabled"];

// ---------------------------------------------------------------- Schreiben

/// Ein Skalar so ausgeben, dass er beim Lesen wieder derselbe ist.
///
/// Anführungszeichen nur, wenn nötig — ein Plan soll lesbar bleiben.
fn scalar(value: &str) -> String {
    let needs_quotes = value.is_empty()
        || value.trim() != value
        || value.starts_with(['#', '-', '?', ':', '&', '*', '!', '|', '>', '\'', '"', '%', '@', '`', '[', '{'])
        || value.contains(": ")
        || value.ends_with(':')
        || value.contains(" #")
        || matches!(
            value.to_ascii_lowercase().as_str(),
            "true" | "false" | "null" | "~" | "yes" | "no" | "on" | "off"
        )
        || value.parse::<f64>().is_ok();

    if needs_quotes {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

/// Mehrzeiliges als Blockstring, einzeiliges als Skalar.
fn text_field(key: &str, value: &str, indent: &str) -> String {
    if value.contains('\n') {
        let mut out = format!("{indent}{key}: |\n");
        for line in value.lines() {
            out.push_str(&format!("{indent}  {line}\n"));
        }
        out
    } else {
        format!("{indent}{key}: {}\n", scalar(value))
    }
}

pub fn to_yaml(plan: &WorkoutPlan) -> String {
    let mut out = String::from("# WOPlanner Trainingsplan\n");
    out.push_str(&format!("version: {FORMAT_VERSION}\n"));
    out.push_str(&text_field("name", &plan.name, ""));
    if let Some(desc) = plan.description.as_deref().filter(|d| !d.trim().is_empty()) {
        out.push_str(&text_field("description", desc, ""));
    }

    out.push_str("schedule:\n");
    out.push_str(&format!("  type: {}\n", scalar(&plan.schedule_type)));
    match plan.schedule_day {
        Some(day) => out.push_str(&format!("  day: {day}\n")),
        None => out.push_str("  day: null\n"),
    }

    out.push_str("exercises:\n");
    for ex in &plan.exercises {
        out.push_str(&format!("  - name: {}\n", scalar(&ex.name)));
        if let Some(instr) = ex.instructions.as_deref().filter(|s| !s.trim().is_empty()) {
            out.push_str(&text_field("instructions", instr, "    "));
        }
        if let Some(url) = ex.video_url.as_deref().filter(|s| !s.trim().is_empty()) {
            out.push_str(&format!("    video_url: {}\n", scalar(url)));
        }
        out.push_str(&format!("    sets: {}\n", ex.sets));
        if let Some(weight) = ex.weight {
            out.push_str(&format!("    weight: {weight}\n"));
        }
        if let Some(notes) = ex.notes.as_deref().filter(|s| !s.trim().is_empty()) {
            out.push_str(&text_field("notes", notes, "    "));
        }
    }
    out
}

/// Dateiname aus dem Plannamen: kleingeschrieben, alles Fremde zu `-`.
pub fn filename_for(name: &str) -> String {
    // Aufeinanderfolgende Trennzeichen zusammenfalten — ein einzelnes
    // replace("--", "-") lässt bei "Leg / Core" einen doppelten Strich stehen.
    let mut slug = String::new();
    for c in name.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c);
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug = slug.trim_matches('-');
    if slug.is_empty() { "workout.yaml".into() } else { format!("{slug}.yaml") }
}

// ------------------------------------------------------------------- Lesen

/// Eine Zeile, die Inhalt trägt: Einrückung, Text, ursprüngliche Zeilennummer.
struct Line<'a> {
    indent: usize,
    text: &'a str,
    number: usize,
}

fn scan(input: &str) -> Result<Vec<Line<'_>>, ParseError> {
    let mut lines = Vec::new();
    for (index, raw) in input.lines().enumerate() {
        let number = index + 1;
        if raw.contains('\t') {
            return err(number, "tabs are not allowed for indentation, use spaces");
        }
        if raw.trim().is_empty() || raw.trim_start().starts_with('#') {
            continue;
        }
        if raw.trim() == "---" || raw.trim() == "..." {
            return err(number, "document markers are not supported, the file holds exactly one plan");
        }
        lines.push(Line {
            indent: raw.len() - raw.trim_start().len(),
            text: raw.trim_end(),
            number,
        });
    }
    Ok(lines)
}

/// `key: value` trennen. Gibt (key, rest) zurück; rest kann leer sein.
fn split_pair(text: &str, number: usize) -> Result<(&str, &str), ParseError> {
    match text.find(':') {
        Some(pos) => {
            let key = text[..pos].trim();
            let value = text[pos + 1..].trim();
            if key.is_empty() {
                return err(number, "missing key before ':'");
            }
            Ok((key, value))
        }
        None => err(number, format!("expected 'key: value', found '{text}'")),
    }
}

/// Anführungszeichen auflösen; `null`/leer wird zu None.
fn unquote(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value == "null" || value == "~" {
        return None;
    }
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        let inner = &value[1..value.len() - 1];
        return Some(inner.replace("\\\"", "\"").replace("\\\\", "\\"));
    }
    if value.len() >= 2 && value.starts_with('\'') && value.ends_with('\'') {
        return Some(value[1..value.len() - 1].replace("''", "'"));
    }
    Some(value.to_string())
}

/// Blockstring einsammeln: alle Folgezeilen, die tiefer eingerückt sind.
fn block_scalar(lines: &[Line<'_>], start: usize, parent_indent: usize) -> (String, usize) {
    let mut collected: Vec<&str> = Vec::new();
    let mut index = start;
    let mut block_indent = None;
    while index < lines.len() && lines[index].indent > parent_indent {
        let indent = *block_indent.get_or_insert(lines[index].indent);
        let text = lines[index].text;
        let stripped = if text.len() >= indent { &text[indent..] } else { text.trim_start() };
        collected.push(stripped);
        index += 1;
    }
    (collected.join("\n"), index)
}

fn parse_number(value: &str, number: usize, field: &str) -> Result<f64, ParseError> {
    value
        .trim()
        .parse::<f64>()
        .map_err(|_| ParseError { line: number, message: format!("{field} must be a number, found '{value}'") })
}

pub fn from_yaml(input: &str) -> Result<WorkoutPlan, ParseError> {
    if input.len() > MAX_INPUT_BYTES {
        return err(0, format!("input is larger than {} KB", MAX_INPUT_BYTES / 1024));
    }
    if input.contains('\u{0}') {
        return err(0, "input contains a null byte");
    }

    let lines = scan(input)?;
    if lines.is_empty() {
        return err(0, "input is empty");
    }

    let mut version = None;
    let mut name = None;
    let mut description = None;
    let mut schedule_type = "manual".to_string();
    let mut schedule_day = None;
    let mut exercises: Vec<PlanExercise> = Vec::new();
    let mut seen_exercises = false;

    let mut i = 0;
    while i < lines.len() {
        let line = &lines[i];
        if line.indent != 0 {
            return err(line.number, "unexpected indentation at top level");
        }
        let (key, value) = split_pair(line.text, line.number)?;
        i += 1;

        match key {
            "version" => {
                version = Some(parse_number(value, line.number, "version")? as i64);
            }
            "name" | "description" => {
                let text = if value == "|" || value == "|-" {
                    let (block, next) = block_scalar(&lines, i, 0);
                    i = next;
                    Some(block)
                } else {
                    unquote(value)
                };
                if key == "name" { name = text } else { description = text }
            }
            "schedule" => {
                if !value.is_empty() {
                    return err(line.number, "schedule must be a block with 'type' and 'day'");
                }
                while i < lines.len() && lines[i].indent > 0 {
                    let sub = &lines[i];
                    let (sub_key, sub_value) = split_pair(sub.text, sub.number)?;
                    match sub_key {
                        "type" => {
                            let t = unquote(sub_value).unwrap_or_else(|| "manual".into());
                            if !SCHEDULE_TYPES.contains(&t.as_str()) {
                                return err(
                                    sub.number,
                                    format!("unknown schedule type '{t}', expected one of {}", SCHEDULE_TYPES.join(", ")),
                                );
                            }
                            schedule_type = t;
                        }
                        "day" => {
                            schedule_day = match unquote(sub_value) {
                                None => None,
                                Some(raw) => {
                                    let day = parse_number(&raw, sub.number, "schedule day")? as i64;
                                    if !(0..=6).contains(&day) {
                                        return err(sub.number, "schedule day must be between 0 (Sunday) and 6");
                                    }
                                    Some(day)
                                }
                            };
                        }
                        other => return err(sub.number, format!("unknown field 'schedule.{other}'")),
                    }
                    i += 1;
                }
            }
            "exercises" => {
                seen_exercises = true;
                if !value.is_empty() {
                    return err(line.number, "exercises must be a block list, one '- name:' per exercise");
                }
                while i < lines.len() && lines[i].indent > 0 {
                    let item = &lines[i];
                    let stripped = item
                        .text
                        .trim_start()
                        .strip_prefix("- ")
                        .ok_or(ParseError {
                            line: item.number,
                            message: format!("expected a list item starting with '- ', found '{}'", item.text.trim()),
                        })?;
                    let item_indent = item.indent;
                    i += 1;

                    let mut fields: Vec<(&str, String, usize)> = Vec::new();
                    let (first_key, first_value) = split_pair(stripped, item.number)?;
                    fields.push((first_key, first_value.to_string(), item.number));

                    while i < lines.len() && lines[i].indent > item_indent {
                        let field = &lines[i];
                        let (field_key, field_value) = split_pair(field.text, field.number)?;
                        i += 1;
                        if field_value == "|" || field_value == "|-" {
                            let (block, next) = block_scalar(&lines, i, field.indent);
                            i = next;
                            fields.push((field_key, block, field.number));
                        } else {
                            fields.push((field_key, field_value.to_string(), field.number));
                        }
                    }

                    exercises.push(build_exercise(&fields, exercises.len() + 1)?);
                    if exercises.len() > MAX_EXERCISES {
                        return err(item.number, format!("a plan holds at most {MAX_EXERCISES} exercises"));
                    }
                }
            }
            other => return err(line.number, format!("unknown field '{other}'")),
        }
    }

    // --- Prüfungen, bevor irgendetwas geschrieben wird
    match version {
        None => return err(0, "missing 'version: 1' — every plan states its format version"),
        Some(v) if v != FORMAT_VERSION => {
            return err(0, format!("unsupported format version {v}, this build reads version {FORMAT_VERSION}"));
        }
        _ => {}
    }

    let name = name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
        .ok_or(ParseError { line: 0, message: "missing plan name".into() })?;
    if name.chars().count() > MAX_NAME_LEN {
        return err(0, format!("plan name is longer than {MAX_NAME_LEN} characters"));
    }
    if !seen_exercises || exercises.is_empty() {
        return err(0, "a plan needs at least one exercise");
    }
    if schedule_type != "weekly" {
        schedule_day = None;
    }

    Ok(WorkoutPlan { name, description, schedule_type, schedule_day, exercises })
}

fn build_exercise(fields: &[(&str, String, usize)], position: usize) -> Result<PlanExercise, ParseError> {
    let mut name = None;
    let mut instructions = None;
    let mut video_url = None;
    let mut sets = None;
    let mut weight = None;
    let mut notes = None;

    for (key, value, number) in fields {
        match *key {
            "name" => name = unquote(value),
            "instructions" => instructions = unquote(value),
            "video_url" => video_url = unquote(value),
            "notes" => notes = unquote(value),
            "sets" => {
                let parsed = parse_number(value, *number, &format!("exercise {position}: sets"))? as i64;
                if !(1..=MAX_SETS).contains(&parsed) {
                    return err(*number, format!("exercise {position}: sets must be between 1 and {MAX_SETS}"));
                }
                sets = Some(parsed);
            }
            "weight" => {
                weight = match unquote(value) {
                    None => None,
                    Some(raw) => {
                        let parsed = parse_number(&raw, *number, &format!("exercise {position}: weight"))?;
                        if parsed < 0.0 {
                            return err(*number, format!("exercise {position}: weight cannot be negative"));
                        }
                        Some(parsed)
                    }
                };
            }
            other => {
                return err(*number, format!("exercise {position}: unknown field '{other}'"));
            }
        }
    }

    let name = name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
        .ok_or(ParseError { line: fields[0].2, message: format!("exercise {position}: missing name") })?;
    if name.chars().count() > MAX_NAME_LEN {
        return err(fields[0].2, format!("exercise {position}: name is longer than {MAX_NAME_LEN} characters"));
    }

    Ok(PlanExercise {
        name,
        instructions,
        video_url,
        sets: sets.unwrap_or(3),
        weight,
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> WorkoutPlan {
        WorkoutPlan {
            name: "Push Day".into(),
            description: Some("Brust, Schultern, Trizeps".into()),
            schedule_type: "weekly".into(),
            schedule_day: Some(3),
            exercises: vec![
                PlanExercise {
                    name: "Bench Press".into(),
                    instructions: Some("Auf die Bank legen.\nExplosiv drücken.".into()),
                    video_url: Some("https://www.youtube.com/watch?v=abc".into()),
                    sets: 4,
                    weight: Some(80.0),
                    notes: Some("Aufwärmen mit 60kg".into()),
                },
                PlanExercise {
                    name: "Pull-up".into(),
                    instructions: None,
                    video_url: None,
                    sets: 3,
                    weight: None,
                    notes: None,
                },
            ],
        }
    }

    #[test]
    fn round_trip_keeps_every_field() {
        let plan = sample();
        let parsed = from_yaml(&to_yaml(&plan)).expect("round trip must parse");
        assert_eq!(parsed, plan);
    }

    #[test]
    fn export_carries_no_ids_or_timestamps() {
        let yaml = to_yaml(&sample());
        for forbidden in ["id:", "user_id", "created_at", "updated_at", "position:"] {
            assert!(!yaml.contains(forbidden), "export must not contain {forbidden}");
        }
    }

    #[test]
    fn minimal_plan_parses() {
        let yaml = "version: 1\nname: Leg Day\nexercises:\n  - name: Squat\n    sets: 5\n";
        let plan = from_yaml(yaml).expect("minimal plan must parse");
        assert_eq!(plan.name, "Leg Day");
        assert_eq!(plan.schedule_type, "manual");
        assert_eq!(plan.exercises.len(), 1);
        assert_eq!(plan.exercises[0].sets, 5);
        assert_eq!(plan.exercises[0].weight, None);
    }

    #[test]
    fn comments_and_blank_lines_are_ignored() {
        let yaml = "# a plan\n\nversion: 1\nname: Leg Day\n\nexercises:\n  # the only one\n  - name: Squat\n    sets: 3\n";
        assert!(from_yaml(yaml).is_ok());
    }

    #[test]
    fn missing_version_is_rejected() {
        let yaml = "name: Leg Day\nexercises:\n  - name: Squat\n    sets: 3\n";
        let error = from_yaml(yaml).unwrap_err();
        assert!(error.message.contains("version"), "got: {error}");
    }

    #[test]
    fn unsupported_version_is_rejected() {
        let yaml = "version: 2\nname: Leg Day\nexercises:\n  - name: Squat\n    sets: 3\n";
        let error = from_yaml(yaml).unwrap_err();
        assert!(error.message.contains("version 2"), "got: {error}");
    }

    #[test]
    fn error_names_the_exercise_position() {
        let yaml = "version: 1\nname: Leg Day\nexercises:\n  - name: Squat\n    sets: 3\n  - name: Lunge\n    sets: 99\n";
        let error = from_yaml(yaml).unwrap_err();
        assert!(error.message.contains("exercise 2"), "got: {error}");
        assert!(error.message.contains("between 1 and 50"), "got: {error}");
    }

    #[test]
    fn negative_weight_is_rejected() {
        let yaml = "version: 1\nname: X\nexercises:\n  - name: Squat\n    sets: 3\n    weight: -5\n";
        assert!(from_yaml(yaml).unwrap_err().message.contains("negative"));
    }

    #[test]
    fn unknown_schedule_type_is_rejected() {
        let yaml = "version: 1\nname: X\nschedule:\n  type: sometimes\nexercises:\n  - name: Squat\n    sets: 3\n";
        let error = from_yaml(yaml).unwrap_err();
        assert!(error.message.contains("unknown schedule type"), "got: {error}");
    }

    #[test]
    fn schedule_day_only_applies_to_weekly() {
        let yaml = "version: 1\nname: X\nschedule:\n  type: rotation\n  day: 3\nexercises:\n  - name: Squat\n    sets: 3\n";
        assert_eq!(from_yaml(yaml).unwrap().schedule_day, None);
    }

    #[test]
    fn tabs_are_rejected_with_a_line_number() {
        let yaml = "version: 1\nname: X\nexercises:\n\t- name: Squat\n";
        let error = from_yaml(yaml).unwrap_err();
        assert_eq!(error.line, 4);
        assert!(error.message.contains("tabs"), "got: {error}");
    }

    #[test]
    fn unknown_field_is_rejected_rather_than_dropped() {
        let yaml = "version: 1\nname: X\nauthor: someone\nexercises:\n  - name: Squat\n    sets: 3\n";
        assert!(from_yaml(yaml).unwrap_err().message.contains("unknown field"));
    }

    #[test]
    fn plan_without_exercises_is_rejected() {
        let yaml = "version: 1\nname: X\nexercises:\n";
        assert!(from_yaml(yaml).unwrap_err().message.contains("at least one exercise"));
    }

    #[test]
    fn oversized_input_is_rejected_before_parsing() {
        let yaml = "a".repeat(MAX_INPUT_BYTES + 1);
        assert!(from_yaml(&yaml).unwrap_err().message.contains("larger than"));
    }

    #[test]
    fn quoted_values_survive_the_round_trip() {
        let plan = WorkoutPlan {
            name: "3x5: heavy \"day\"".into(),
            description: None,
            schedule_type: "manual".into(),
            schedule_day: None,
            exercises: vec![PlanExercise {
                name: "Squat".into(),
                instructions: None,
                video_url: None,
                sets: 3,
                weight: None,
                notes: Some("# not a comment".into()),
            }],
        };
        assert_eq!(from_yaml(&to_yaml(&plan)).unwrap(), plan);
    }

    #[test]
    fn filenames_are_derived_from_the_plan_name() {
        assert_eq!(filename_for("Push Day"), "push-day.yaml");
        assert_eq!(filename_for("Leg / Core!"), "leg-core.yaml");
        assert_eq!(filename_for("???"), "workout.yaml");
    }
}
