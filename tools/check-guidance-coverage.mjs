import fs from 'node:fs';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
import path from 'node:path';

const digest=text=>createHash('sha256').update(text).digest('hex');
const inventory=JSON.parse(fs.readFileSync('docs/agents/guidance-coverage.json','utf8'));
const source=execFileSync('git',['show',inventory.source.commit+':'+inventory.source.path],{encoding:'utf8'}).replace(/\r\n/g,'\n');
assert.equal(digest(source),inventory.source.sha256,'Frozen source changed');
const lines=source.split('\n'),root=fs.readFileSync('AGENTS.md','utf8').replace(/\r\n/g,'\n');
let next=1;
for(const rule of inventory.rules){
 assert.equal(rule.sourceStart,next,'Coverage has a gap or overlap: '+rule.id);
 const text=lines.slice(rule.sourceStart-1,rule.sourceEnd).join('\n').trim();
 assert.equal(digest(text),rule.sha256,'Rule identity changed: '+rule.id);
 const home=fs.readFileSync(rule.home,'utf8').replace(/\r\n/g,'\n');
 const normalized=rule.normalization==='format-check'?text.replace('cargo fmt                      # format (must produce no diff)','cargo fmt --check              # format (must produce no diff)'):
 rule.normalization==='relative-links'?text.replace(/\]\((?!https?:|#)([^)]+)\)/g,(_,target)=>']('+path.posix.relative(path.posix.dirname(rule.home),target)+')'):text;
 assert.ok(home.includes(normalized)||(!process.argv.includes('--final')&&rule.home==='AGENTS.md'&&home.includes(text)),'Missing rule text: '+rule.id+' in '+rule.home);
 assert.ok(rule.trigger,'Missing read trigger: '+rule.id);
 if(rule.home!=='AGENTS.md'){
  assert.ok(home.includes('id="'+rule.id+'"'),'Missing map anchor: '+rule.id);
  if(process.argv.includes('--final'))assert.ok(!root.includes(text),'Extracted text still duplicated: '+rule.id);
  if(process.argv.includes('--final'))assert.ok(root.includes('('+rule.home+')'),'Unreachable reference home: '+rule.home);
 }
 next=rule.sourceEnd+1;
}
assert.equal(next,lines.length+1,'Unmapped source tail');
for(const file of new Set(['docs/agents/guidance-coverage.md',...inventory.rules.map(rule=>rule.home)])){
 const text=fs.readFileSync(file,'utf8');
 for(const match of text.matchAll(/\]\((?!https?:|#)([^)]+)\)/g)){
  const target=path.resolve(path.dirname(file),match[1].split('#')[0]);
  assert.ok(fs.existsSync(target),'Broken local link in '+file+': '+match[1]);
 }
}
console.log(`pass: ${inventory.rules.length} units cover ${lines.length} frozen source lines; ${process.argv.includes('--final')?'final':'transition'} homes intact`);
