import {test} from "node:test";
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import {generateStageWorkflow} from "./generate-release-workflow.mjs";
test("real dist generation is deterministic and matches the committed workflow", () => {
  const first = generateStageWorkflow();
  assert.equal(first, generateStageWorkflow());
  assert.equal(first, readFileSync(new URL("../.github/workflows/release.yml", import.meta.url), "utf8"));
});
