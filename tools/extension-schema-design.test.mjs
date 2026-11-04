import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { parse } from 'smol-toml';

const root = new URL('../tests/fixtures/extension-schema-v1/', import.meta.url);
const bytes = name => readFileSync(new URL(name, root));
const sha = name => createHash('sha256').update(bytes(name)).digest('hex');

test('design schema examples use only the approved optional scalar kinds', () => {
  for (const name of ['schema.toml', 'schema-after.toml']) {
    const schema = parse(bytes(name).toString('utf8'));
    assert.equal(schema.format_version, 1);
    assert.deepEqual(Object.keys(schema).sort(), ['fields', 'format_version']);
    assert.equal(Object.keys(schema.fields).length, 2);
    for (const [key, field] of Object.entries(schema.fields)) {
      assert.match(key, /^x_[a-z][a-z0-9_]{0,47}$/);
      assert.ok(['string', 'enum'].includes(field.type));
      assert.equal(Object.hasOwn(field, 'required'), false);
      if (field.type === 'enum') {
        assert.deepEqual(Object.keys(field).sort(), ['ai_context', 'type', 'values']);
        assert.deepEqual(field.values, ['platform', 'frontend']);
      }
      if (field.type === 'string') {
        assert.deepEqual(Object.keys(field).sort(), ['max_bytes', 'type']);
        assert.equal(field.max_bytes, 2048);
      }
    }
  }
});

test('review-only migration preview binds exact before/after fixture bytes', () => {
  const plan = JSON.parse(bytes('rename-preview.json'));
  assert.equal(plan.kind, 'design-fixture-not-an-executable-cli-plan');
  assert.equal(plan.plan_format, 1);
  assert.equal(plan.schema_format, 1);
  assert.equal(plan.schema_before_sha256, sha('schema.toml'));
  assert.equal(plan.schema_after_sha256, sha('schema-after.toml'));
  assert.equal(plan.changes.length, 1);
  const change = plan.changes[0];
  assert.equal(change.before_sha256, sha('before.md'));
  assert.equal(change.after_sha256, sha('after.md'));
  assert.equal(change.from, 'x_team');
  assert.equal(change.to, 'x_group');
  assert.equal(change.raw_value, ' "platform"');
  assert.equal(bytes('before.md').toString().replace('\nx_team:', '\nx_group:'), bytes('after.md').toString());
});
