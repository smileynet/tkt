import {test} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,symlinkSync,lstatSync,readFileSync,rmSync} from 'node:fs';
import {join,relative} from 'node:path';
import {tmpdir} from 'node:os';
import {migrateLegacySkills} from './migrate-legacy-skills.mjs';
function fixture(t) {
  const root=mkdtempSync(join(tmpdir(),'tkt-legacy-')),home=join(root,'home'),source=join(root,'old','skills','tkt');
  mkdirSync(source,{recursive:true}); writeFileSync(join(source,'SKILL.md'),'---\nname: tkt\n---\nPrevious release');
  for(const harness of ['.kiro','.claude','.codex'])mkdirSync(join(home,harness,'skills'),{recursive:true});
  t.after(()=>rmSync(root,{recursive:true,force:true}));
  const link=(h,target=source)=>symlinkSync(target,join(home,h,'skills','tkt'),process.platform==='win32'?'junction':'dir');
  return {home,source,link};
}
test('legacy migration previews all three links without changing them',t=>{
  const f=fixture(t);for(const h of ['.kiro','.claude','.codex'])f.link(h);
  const report=migrateLegacySkills(f);assert.equal(report.status,'pass');assert.equal(report.rows.filter(r=>r.action==='remove-link').length,3);
  for(const h of ['.kiro','.claude','.codex'])assert.ok(lstatSync(join(f.home,h,'skills','tkt')).isSymbolicLink());
});
test('legacy migration removes only matching links and can run twice',t=>{
  const f=fixture(t);for(const h of ['.kiro','.claude','.codex'])f.link(h);
  writeFileSync(join(f.home,'.codex','skills','unrelated'),'user bytes');
  assert.equal(migrateLegacySkills({...f,execute:true}).removed,3);
  assert.equal(migrateLegacySkills({...f,execute:true}).removed,0);
  assert.equal(readFileSync(join(f.source,'SKILL.md'),'utf8'),'---\nname: tkt\n---\nPrevious release');
  assert.equal(readFileSync(join(f.home,'.codex','skills','unrelated'),'utf8'),'user bytes');
});
test('legacy migration preserves foreign directories and refuses a partial cleanup',t=>{
  const f=fixture(t);f.link('.kiro');mkdirSync(join(f.home,'.codex','skills','tkt'));writeFileSync(join(f.home,'.codex','skills','tkt','user'),'foreign');
  const report=migrateLegacySkills({...f,execute:true});assert.equal(report.status,'fail');assert.equal(report.removed,0);assert.ok(lstatSync(join(f.home,'.kiro','skills','tkt')).isSymbolicLink());
  assert.equal(readFileSync(join(f.home,'.codex','skills','tkt','user'),'utf8'),'foreign');
});
test('legacy migration preserves links to another source',t=>{
  const f=fixture(t),other=join(f.home,'other');mkdirSync(other);f.link('.codex',other);
  const report=migrateLegacySkills({...f,execute:true});assert.equal(report.status,'fail');assert.ok(lstatSync(join(f.home,'.codex','skills','tkt')).isSymbolicLink());
});
test('legacy migration recognizes a dangling link from the explicitly supplied source',t=>{
  const f=fixture(t);f.link('.codex');rmSync(f.source,{recursive:true});
  const report=migrateLegacySkills({...f,execute:true});assert.equal(report.status,'pass');assert.equal(report.removed,1);
});
test('legacy migration resolves a relative link without following the target', {skip:process.platform==='win32'},t=>{
  const f=fixture(t);f.link('.codex',relative(join(f.home,'.codex','skills'),f.source));
  assert.equal(migrateLegacySkills({...f,execute:true}).removed,1);
});
test('legacy migration refuses roots redirected outside the selected home',t=>{
  const f=fixture(t),outside=join(f.source,'outside');mkdirSync(outside);rmSync(join(f.home,'.codex'),{recursive:true});symlinkSync(outside,join(f.home,'.codex'),process.platform==='win32'?'junction':'dir');mkdirSync(join(outside,'skills'));symlinkSync(f.source,join(outside,'skills','tkt'),process.platform==='win32'?'junction':'dir');
  const report=migrateLegacySkills({...f,execute:true});assert.equal(report.status,'fail');assert.equal(report.removed,0);assert.ok(lstatSync(join(outside,'skills','tkt')).isSymbolicLink());
});
