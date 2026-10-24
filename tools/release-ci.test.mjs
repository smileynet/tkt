import { test } from "node:test";
import assert from "node:assert/strict";
import { cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { tmpdir } from "node:os";
import { join } from "node:path";
import * as tar from "tar";
import { parse } from "yaml";
import { ciIdentity, sealStage } from "./release-ci.mjs";
import { generateStageWorkflow, stageWorkflow } from "./generate-release-workflow.mjs";
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
function fixture() {
  const root = mkdtempSync(join(tmpdir(), "tkt CI seal ")), repo = join(root, "repo"), input = join(root, "input"), output = join(root, "output");
  mkdirSync(repo); mkdirSync(input);
  for (const file of ["Cargo.toml", "Cargo.lock", ".claude-plugin", ".codex-plugin", "plugin.json"]) cpSync(new URL(`../${file}`, import.meta.url), join(repo, file), { recursive: true });
  const git = (...args) => execFileSync("git", ["-C", repo, ...args], { encoding: "utf8", stdio: "pipe" }).trim();
  git("init"); git("add", "."); git("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "-c", `core.hooksPath=${join(root, "no-hooks")}`, "commit", "-m", "Candidate");
  const env = { EXPECTED_CANDIDATE_SHA: git("rev-parse", "HEAD"), GITHUB_SHA: git("rev-parse", "HEAD"), GITHUB_REPOSITORY: "smileynet/tkt", GITHUB_RUN_ID: "123", GITHUB_RUN_ATTEMPT: "2" };
  const targets = ["aarch64-apple-darwin", "aarch64-unknown-linux-gnu", "x86_64-apple-darwin", "x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc"];
  const artifacts = Object.fromEntries(targets.map((target) => [`tkt-${target}.zip`, { kind: "executable-zip", target_triples: [target] }]));
  const plan = { dist_version: "0.32.0", announcement_tag: "v0.4.0", releases: [{ app_name: "tkt", app_version: "0.4.0", artifacts: Object.keys(artifacts) }], artifacts };
  const writePlan = () => writeFileSync(join(input, "plan-dist-manifest.json"), JSON.stringify(plan)); writePlan();
  for (const name of Object.keys(artifacts)) writeFileSync(join(input, name), `sealed ${name}`);
  const source = join(root, "crate-source"), prefix = "tkt-0.4.0"; mkdirSync(join(source, prefix), { recursive: true });
  const crate = (sha = env.GITHUB_SHA) => { writeFileSync(join(source, prefix, ".cargo_vcs_info.json"), JSON.stringify({ git: { sha1: sha }, path_in_vcs: "" })); tar.create({ file: join(input, `${prefix}.crate`), cwd: source, sync: true, gzip: true }, [`${prefix}/.cargo_vcs_info.json`]); }; crate();
  return { repo, input, output, env, plan, writePlan, crate, run: () => sealStage({ repo, input, output, env }), clean: () => rmSync(root, { recursive: true, force: true }) };
}
test("real dist generation is deterministic and retains only read-only manual staging jobs", () => {
  const first = generateStageWorkflow(), second = generateStageWorkflow(); assert.equal(first, second);
  assert.equal(first, readFileSync(new URL("../.github/workflows/release.yml", import.meta.url), "utf8"));
  const workflow = parse(first); assert.deepEqual(workflow.permissions, { contents: "read" }); assert.deepEqual(Object.keys(workflow.on), ["workflow_dispatch"]);
  assert.deepEqual(Object.keys(workflow.jobs), ["plan", "build-local-artifacts", "build-global-artifacts", "build-crate", "seal-stage"]);
  assert.doesNotMatch(first, /dist host|gh release|cargo publish|git push|secrets\./);
  assert.match(workflow.jobs.plan.steps.find((step) => step.id === "plan").run, /dist plan/);
  assert.match(workflow.jobs["build-local-artifacts"].steps.find((step) => step.name === "Build artifacts").run, /dist build/);
  assert.throws(() => stageWorkflow("jobs: { plan: {} }"), /topology/);
});
test("CI seal binds complete built files to candidate and run without rebuilding", () => {
  const f = fixture(); try {
    assert.equal(ciIdentity(f.repo, f.env).candidateSha, f.env.GITHUB_SHA);
    const result = f.run(); assert.equal(result.files.length, 6); assert.equal(result.runAttempt, "2"); assert.equal(result.publicationPerformed, false);
    for (const file of result.files) assert.equal(sha256(readFileSync(join(f.output, file.name))), file.sha256);
    assert.throws(f.run, /already exists/);
  } finally { f.clean(); }
});
test("CI seal refuses missing/foreign/empty files, wrong crate commit and plan target omission", () => {
  for (const mutation of [
    (f) => rmSync(join(f.input, Object.keys(f.plan.artifacts)[0])),
    (f) => writeFileSync(join(f.input, "foreign.txt"), "unreviewed"),
    (f) => writeFileSync(join(f.input, Object.keys(f.plan.artifacts)[0]), ""),
    (f) => f.crate("a".repeat(40)),
    (f) => { const name = f.plan.releases[0].artifacts.pop(); delete f.plan.artifacts[name]; f.writePlan(); },
    (f) => { f.env.GITHUB_SHA = "a".repeat(40); },
    (f) => { f.env.GITHUB_REPOSITORY = "attacker/tkt"; },
  ]) { const f = fixture(); try { mutation(f); assert.throws(f.run); } finally { f.clean(); } }
});

test("CI seal validates sidecar checksum syntax and exact bytes before output", () => {
  const f = fixture(); try {
    const parent = Object.keys(f.plan.artifacts)[0], name = `${parent}.sha256`, digest = sha256(readFileSync(join(f.input, parent)));
    f.plan.artifacts[name] = { kind: "checksum" }; f.plan.releases[0].artifacts.push(name); f.writePlan();
    writeFileSync(join(f.input, name), `${digest}garbage`); assert.throws(f.run, /checksum differs/);
    writeFileSync(join(f.input, name), `${digest}  ${parent}\n`); assert.equal(f.run().status, "pass");
  } finally { f.clean(); }
});

test("Windows dist bootstrap retains the native PowerShell execution shell", () => {
  const workflow = parse(readFileSync(new URL("../.github/workflows/release.yml", import.meta.url), "utf8"));
  const job = workflow.jobs["build-local-artifacts"], install = job.steps.find((step) => step.name === "Install dist");
  const windowsShell = install.shell ?? job.defaults?.run?.shell ?? workflow.defaults?.run?.shell ?? "pwsh";
  assert.equal(windowsShell, "pwsh", "PowerShell irm/iex bootstrap must not execute in bash");
});

test("CI intermediate uploads and downloads are isolated by run attempt", () => {
  const workflow = parse(readFileSync(new URL("../.github/workflows/release.yml", import.meta.url), "utf8"));
  for (const [name, job] of Object.entries(workflow.jobs)) for (const step of job.steps) {
    if (name === "seal-stage" && step.uses?.startsWith("actions/upload-artifact@")) continue;
    if (step.uses?.startsWith("actions/upload-artifact@") || step.uses?.startsWith("actions/download-artifact@")) {
      assert.ok((step.with.name ?? step.with.pattern).includes("attempt-${{ github.run_attempt }}"), `${name}: ${step.with.name ?? step.with.pattern}`);
    }
  }
});
