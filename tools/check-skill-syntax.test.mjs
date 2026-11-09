import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { checkPack } from "./check-skill-syntax.mjs";
import { removeTree } from "./test-fixtures.mjs";


function fixture(t, body) {
  const pack = mkdtempSync(join(tmpdir(), "tkt-skill-syntax-"));
  t.after(() => removeTree(pack));
  mkdirSync(join(pack, "skills/example"), { recursive: true });
  writeFileSync(join(pack, "skills/example/SKILL.md"), `---\nname: example\ndescription: Example fixture\n---\n${body}`);
  return pack;
}

test("skill body limit accepts 24000 bytes and rejects larger or multibyte overflow", t => {
  assert.equal(checkPack(fixture(t, "x".repeat(24000))).status, "pass");
  const over = checkPack(fixture(t, "x".repeat(24001)));
  assert.equal(over.status, "fail");
  assert.match(over.errors.join("\n"), /body is 24001 bytes; at most 24000/);
  const multibyte = checkPack(fixture(t, "é".repeat(12001)));
  assert.equal(multibyte.status, "fail");
  assert.match(multibyte.errors.join("\n"), /body is 24002 bytes; at most 24000/);
});

test("repository skills satisfy static syntax and body limits", () => {
  const root = fileURLToPath(new URL("../", import.meta.url));
  const result = checkPack(root);
  assert.equal(result.status, "pass", result.errors.join("\n"));
  assert.equal(result.metrics.skills, 4);
});
