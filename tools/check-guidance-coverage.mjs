import fs from 'node:fs';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
import path from 'node:path';

const digest=text=>createHash('sha256').update(text).digest('hex');
const inventory=JSON.parse(fs.readFileSync('docs/agents/guidance-coverage.json','utf8'));
const final=process.argv.includes('--final')||inventory.phase==='final';
const source=execFileSync('git',['show',inventory.source.commit+':'+inventory.source.path],{encoding:'utf8'}).replace(/\r\n/g,'\n');
assert.equal(digest(source),inventory.source.sha256,'Frozen source changed');
const lines=source.split('\n'),root=fs.readFileSync('AGENTS.md','utf8').replace(/\r\n/g,'\n');
let next=1;
for(const rule of inventory.rules){
 assert.equal(rule.sourceStart,next,'Coverage has a gap or overlap: '+rule.id);
 const text=lines.slice(rule.sourceStart-1,rule.sourceEnd).join('\n').trim();
 assert.equal(digest(text),rule.sha256,'Rule identity changed: '+rule.id);
 const home=fs.readFileSync(rule.home,'utf8').replace(/\r\n/g,'\n');
 let normalized=rule.normalization==='format-check'?text.replace('cargo fmt                      # format (must produce no diff)','cargo fmt --check              # format (must produce no diff)'):
 rule.normalization==='relative-links'?text.replace(/\]\((?!https?:|#)([^)]+)\)/g,(_,target)=>']('+path.posix.relative(path.posix.dirname(rule.home),target)+')'):text;
 if(rule.amendment){
  execFileSync('git',['merge-base','--is-ancestor',rule.amendment.commit,'HEAD']);
  const deleted=execFileSync('git',['diff','--name-status',rule.amendment.commit+'^',rule.amendment.commit,'--','.github/workflows/ci.yml'],{encoding:'utf8'});
  assert.equal(deleted.trim(),'D\t.github/workflows/ci.yml','Policy amendment must match owner workflow removal');
  assert.ok(normalized.includes(rule.amendment.from),'Amendment no longer matches frozen text');
  normalized=normalized.replace(rule.amendment.from,rule.amendment.to);
 }
 assert.ok(home.includes(normalized)||(!final&&rule.home==='AGENTS.md'&&home.includes(text)),'Missing rule text: '+rule.id+' in '+rule.home);
 assert.ok(rule.trigger,'Missing read trigger: '+rule.id);
 if(rule.home!=='AGENTS.md'){
  assert.ok(home.includes('id="'+rule.id+'"'),'Missing map anchor: '+rule.id);
  if(final)assert.ok(!root.includes(text),'Extracted text still duplicated: '+rule.id);
  if(final)assert.ok(root.includes('('+rule.home+')'),'Unreachable reference home: '+rule.home);
 }
 next=rule.sourceEnd+1;
}
assert.equal(next,lines.length+1,'Unmapped source tail');
const report=fs.readFileSync('docs/agents/guidance-coverage.md','utf8');
assert.equal((report.match(/^\| rule-\d+ \|/gm)||[]).length,inventory.rules.length,'Human table row count differs');
for(const rule of inventory.rules){
 const home=rule.home==='AGENTS.md'?'../../AGENTS.md':rule.home.replace('docs/agents/','')+'#'+rule.id;
 const expected=`| ${rule.id} | [${rule.sourceStart}–${rule.sourceEnd}](https://github.com/smileynet/tkt/blob/${inventory.source.commit}/AGENTS.md#L${rule.sourceStart}-L${rule.sourceEnd}) | ${rule.title.replaceAll('|','/').slice(0,95)} | [${rule.home}](${home}) | ${rule.trigger} |`;
 assert.ok(report.includes(expected),'Human table differs from inventory: '+rule.id);
}
for(const file of new Set(['docs/agents/guidance-coverage.md',...inventory.rules.map(rule=>rule.home)])){
 const text=fs.readFileSync(file,'utf8');
 for(const match of text.matchAll(/\]\((?!https?:|#)([^)]+)\)/g)){
  const target=path.resolve(path.dirname(file),match[1].split('#')[0]);
  assert.ok(fs.existsSync(target),'Broken local link in '+file+': '+match[1]);
 }
}
console.log(`pass: ${inventory.rules.length} units cover ${lines.length} frozen source lines; ${final?'final':'transition'} homes intact`);
