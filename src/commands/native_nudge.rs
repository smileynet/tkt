use std::collections::BTreeSet;
use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result};
use clap::ValueEnum;
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{json, Value};

use super::common::{is_dry_run, is_json_output, is_quiet, tickets_dir};
use crate::effects::{FileEffect, FilePlan, Mode};
use crate::{DomainError, ErrorKind};

#[derive(Clone, Copy, ValueEnum)]
pub enum Target {
    Claude,
    Codex,
    Both,
}
#[derive(Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum Action {
    Install,
    Check,
    Remove,
}

const CONFIG_BYTES: u64 = 1024 * 1024;
const SCRIPT_BYTES: u64 = 65536;
const SCRIPT: &str = include_str!("../../assets/native-write-nudge.mjs");
const _: () = assert!(SCRIPT.len() <= SCRIPT_BYTES as usize);
const SCRIPT_PATH: &str = ".tickets/hooks/native-write-nudge.mjs";
const OWNER_PATH: &str = ".tickets/hooks/native-write-nudge.json";

struct StrictJson(Value);
impl<'de> Deserialize<'de> for StrictJson {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        struct JsonVisitor;
        impl<'de> Visitor<'de> for JsonVisitor {
            type Value = Value;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("JSON without duplicate object keys")
            }
            fn visit_bool<E: de::Error>(self, value: bool) -> std::result::Result<Value, E> {
                Ok(Value::Bool(value))
            }
            fn visit_i64<E: de::Error>(self, value: i64) -> std::result::Result<Value, E> {
                Ok(value.into())
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> std::result::Result<Value, E> {
                Ok(value.into())
            }
            fn visit_f64<E: de::Error>(self, value: f64) -> std::result::Result<Value, E> {
                serde_json::Number::from_f64(value)
                    .map(Value::Number)
                    .ok_or_else(|| E::custom("non-finite JSON number"))
            }
            fn visit_str<E: de::Error>(self, value: &str) -> std::result::Result<Value, E> {
                Ok(Value::String(value.into()))
            }
            fn visit_string<E: de::Error>(self, value: String) -> std::result::Result<Value, E> {
                Ok(Value::String(value))
            }
            fn visit_unit<E: de::Error>(self) -> std::result::Result<Value, E> {
                Ok(Value::Null)
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> std::result::Result<Value, A::Error> {
                let mut values = Vec::new();
                while let Some(StrictJson(value)) = seq.next_element()? {
                    values.push(value);
                }
                Ok(Value::Array(values))
            }
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut map: A,
            ) -> std::result::Result<Value, A::Error> {
                let mut values = serde_json::Map::new();
                while let Some(key) = map.next_key::<String>()? {
                    if values.contains_key(&key) {
                        return Err(de::Error::custom(format!("duplicate object key {key:?}")));
                    }
                    let StrictJson(value) = map.next_value()?;
                    values.insert(key, value);
                }
                Ok(Value::Object(values))
            }
        }
        deserializer.deserialize_any(JsonVisitor).map(Self)
    }
}

fn decimal(token: &str) -> Option<(bool, String, i64)> {
    let negative = token.starts_with('-');
    let token = token.strip_prefix('-').unwrap_or(token);
    let (mantissa, exponent) = match token.split_once(['e', 'E']) {
        Some((value, exponent)) => (value, exponent.parse::<i64>().ok()?),
        None => (token, 0),
    };
    let fraction = mantissa
        .split_once('.')
        .map_or(0, |(_, fraction)| fraction.len());
    let mut digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    digits = digits.trim_start_matches('0').into();
    if digits.is_empty() {
        return Some((negative, "0".into(), 0));
    }
    let mut exponent = exponent.checked_sub(fraction as i64)?;
    while digits.ends_with('0') {
        digits.pop();
        exponent = exponent.checked_add(1)?;
    }
    Some((negative, digits, exponent))
}

fn parse(text: &str) -> Result<Value> {
    let StrictJson(value) = serde_json::from_str::<StrictJson>(text)?;
    // Native JSON is reserialized; reject a numeric token if that would change
    // its exact decimal value (or signed zero), rather than round foreign data.
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => {
                index += 1;
                while bytes[index] != b'"' {
                    index += if bytes[index] == b'\\' { 2 } else { 1 };
                }
                index += 1;
            }
            b'-' | b'0'..=b'9' => {
                let start = index;
                index += 1;
                while index < bytes.len()
                    && matches!(bytes[index], b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E')
                {
                    index += 1;
                }
                let raw = &text[start..index];
                let encoded = serde_json::from_str::<serde_json::Number>(raw)?.to_string();
                let original = decimal(raw);
                if original.is_none() || original != decimal(&encoded) {
                    return Err(conflict("native JSON contains a number whose exact round-trip cannot be verified; merge it manually without rounding"));
                }
            }
            _ => index += 1,
        }
    }
    Ok(value)
}

fn conflict(message: impl Into<String>) -> anyhow::Error {
    DomainError::with_hint(ErrorKind::Conflict, message.into(), "preserve foreign/modified content; inspect the config and owned files before explicit recovery".into()).into()
}
fn directory(path: &Path) -> Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if !metadata.is_dir() || crate::effects::linked(&metadata) => Err(conflict(
            format!("{} is not an unlinked directory", path.display()),
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}
fn read(path: &Path, limit: u64) -> Result<Option<String>> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if !metadata.is_file() || crate::effects::linked(&metadata) => {
            return Err(conflict(format!(
                "{} is not an unlinked regular file",
                path.display()
            )))
        }
        Ok(_) => (),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .with_context(|| format!("reading {}", path.display()))?
        .take(limit + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(conflict(format!(
            "{} exceeds its {limit}-byte input limit",
            path.display()
        )));
    }
    Ok(Some(String::from_utf8(bytes).map_err(|error| {
        DomainError::new(
            ErrorKind::Parse,
            format!("{} is not UTF-8: {error}", path.display()),
        )
    })?))
}
fn group(profile: &str) -> Value {
    let handler = if profile == "claude" {
        json!({"type":"command","command":"node","args":["${CLAUDE_PROJECT_DIR}/.tickets/hooks/native-write-nudge.mjs","claude"],"timeout":3,"onFailure":"continue"})
    } else {
        json!({"type":"command","command":"node \"$(git rev-parse --show-toplevel)/.tickets/hooks/native-write-nudge.mjs\" codex","commandWindows":"powershell.exe -NoLogo -NoProfile -NonInteractive -WindowStyle Hidden -Command \"[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new(); node (Join-Path (git rev-parse --show-toplevel) '.tickets/hooks/native-write-nudge.mjs') codex\"","timeout":3,"additionalContextLimit":256})
    };
    json!({"matcher":if profile=="claude" {"Edit|Write"}else{"^apply_patch$"},"hooks":[handler]})
}
fn claims_script(value: &Value) -> bool {
    match value {
        Value::String(text) => text.contains("native-write-nudge.mjs"),
        Value::Array(values) => values.iter().any(claims_script),
        Value::Object(values) => values.values().any(claims_script),
        _ => false,
    }
}

pub fn run(target: Target, action: Action) -> Result<i32> {
    let dir = tickets_dir()?;
    directory(&dir)?;
    let repo = dir.parent().unwrap().to_path_buf();
    directory(&repo.join(".tickets/hooks"))?;
    let selected: &[&str] = match target {
        Target::Claude => &["claude"],
        Target::Codex => &["codex"],
        Target::Both => &["claude", "codex"],
    };
    for profile in selected {
        directory(&repo.join(format!(".{profile}")))?;
    }
    let owner_before = read(&repo.join(OWNER_PATH), 8192)?;
    let mut owned = BTreeSet::new();
    if let Some(text) = &owner_before {
        let value =
            parse(text).map_err(|error| conflict(format!("invalid ownership record: {error}")))?;
        if value.as_object().map(|v| v.len()) != Some(3)
            || value["kind"] != "tkt-native-write-nudge"
            || value["version"] != 1
        {
            return Err(conflict("unrecognized native-nudge ownership record"));
        }
        let profiles = value["profiles"]
            .as_array()
            .ok_or_else(|| conflict("invalid owned profile list"))?;
        for profile in profiles {
            let name = profile
                .as_str()
                .ok_or_else(|| conflict("invalid owned profile"))?;
            if !["claude", "codex"].contains(&name) || !owned.insert(name.to_string()) {
                return Err(conflict("invalid/duplicate owned profile"));
            }
        }
    }
    let script_before = read(&repo.join(SCRIPT_PATH), SCRIPT_BYTES)?;
    if script_before.is_some()
        && (owner_before.is_none() || script_before.as_deref() != Some(SCRIPT))
    {
        return Err(conflict("native-nudge helper is unowned or modified"));
    }
    let mut effects = Vec::new();
    let mut installed = true;
    for profile in selected {
        let path = repo.join(if *profile == "claude" {
            ".claude/settings.json"
        } else {
            ".codex/hooks.json"
        });
        let before = read(&path, CONFIG_BYTES)?;
        let mut config: Value = match &before {
            Some(text) => parse(text)
                .map_err(|error| conflict(format!("{}: invalid JSON: {error}", path.display())))?,
            None => json!({}),
        };
        let object = config
            .as_object_mut()
            .ok_or_else(|| conflict("native configuration must be a JSON object"))?;
        let hooks = object
            .entry("hooks")
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .ok_or_else(|| conflict("hooks must be an object"))?;
        let groups = hooks
            .entry("PreToolUse")
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .ok_or_else(|| conflict("PreToolUse must be an array"))?;
        let expected = group(profile);
        let positions: Vec<_> = groups
            .iter()
            .enumerate()
            .filter(|(_, value)| **value == expected)
            .map(|(index, _)| index)
            .collect();
        let registered = owned.contains(*profile);
        if positions.len() > 1
            || (registered && positions.len() != 1)
            || (!registered && groups.iter().any(claims_script))
        {
            return Err(conflict(format!(
                "{profile} native nudge is missing, modified, duplicated or unowned"
            )));
        }
        installed &= registered && script_before.is_some();
        match action {
            Action::Check => (),
            Action::Install if !registered => {
                groups.push(expected);
                owned.insert(profile.to_string());
            }
            Action::Remove if registered => {
                groups.remove(positions[0]);
                owned.remove(*profile);
            }
            _ => (),
        }
        if action != Action::Check
            && ((action == Action::Install && !registered)
                || (action == Action::Remove && registered))
        {
            let text = format!("{}\n", serde_json::to_string_pretty(&config)?);
            if text.len() as u64 > CONFIG_BYTES {
                return Err(conflict(format!(
                    "{} would exceed the {CONFIG_BYTES}-byte native config limit",
                    path.display()
                )));
            }
            effects.push(FileEffect::Directory(path.parent().unwrap().into()));
            effects.push(FileEffect::write(path, before, text));
        }
    }
    if action != Action::Check && (owner_before.is_some() || action == Action::Install) {
        let text = format!(
            "{}\n",
            serde_json::to_string_pretty(
                &json!({"kind":"tkt-native-write-nudge","version":1,"profiles":owned})
            )?
        );
        if owner_before.as_deref() != Some(&text) {
            effects.push(FileEffect::Directory(repo.join(".tickets/hooks")));
            effects.push(FileEffect::write(repo.join(OWNER_PATH), owner_before, text));
        }
        if action == Action::Install && script_before.is_none() {
            effects.insert(
                0,
                FileEffect::write(repo.join(SCRIPT_PATH), None, SCRIPT.into()),
            );
            effects.insert(0, FileEffect::Directory(repo.join(".tickets/hooks")));
        }
    }
    let changed = !effects.is_empty();
    FilePlan(effects).execute(Mode::current())?;
    let installed = if is_dry_run() {
        installed
    } else {
        match action {
            Action::Install => true,
            Action::Remove => false,
            Action::Check => installed,
        }
    };
    if is_json_output() {
        println!(
            "{}",
            json!({"ok":action!=Action::Check||installed,"operation":"init_write_nudge","profiles":selected,"action":match action{Action::Install=>"install",Action::Check=>"check",Action::Remove=>"remove"},"installed":installed,"changed":changed,"dry_run":is_dry_run(),"trust_changed":false})
        );
    } else if !is_quiet() {
        println!(
            "{} native write nudge for {}{}; harness trust is unchanged",
            if action == Action::Check {
                if installed {
                    "Installed"
                } else {
                    "Not installed"
                }
            } else {
                match (action, is_dry_run()) {
                    (Action::Install, true) => "Would install",
                    (Action::Install, false) => "Installed",
                    (Action::Remove, true) => "Would remove",
                    (Action::Remove, false) => "Removed",
                    (Action::Check, _) => unreachable!(),
                }
            },
            selected.join(", "),
            if changed { "" } else { " (no changes)" }
        );
    }
    Ok(i32::from(action == Action::Check && !installed))
}
