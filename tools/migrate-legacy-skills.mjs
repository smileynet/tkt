#!/usr/bin/env node
// Remove v0.4 installer symlinks only when they target an explicitly supplied old source.
// Usage: node tools/migrate-legacy-skills.mjs --source OLD_CHECKOUT/skills/tkt [--home DIR] [--execute]
import {lstatSync,readlinkSync,realpathSync,unlinkSync} from 'node:fs';
import {homedir} from 'node:os';
import {basename,dirname,join,resolve,sep} from 'node:path';
import {fileURLToPath} from 'node:url';
function stat(path) {
  try {return lstatSync(path);} catch(error) {if(error.code==='ENOENT')return null;throw error;}
}
function canonical(path) {
  try {return realpathSync(path);} catch(error) {
    if(error.code!=='ENOENT')throw error;
    return join(canonical(dirname(path)),basename(path));
  }
}
export function migrateLegacySkills({source,home=homedir(),execute=false}) {
  if(typeof source!=='string'||!source.trim())throw Error('An explicit --source OLD_CHECKOUT/skills/tkt is required');
  source=canonical(resolve(source));home=realpathSync(resolve(home));
  const rows=['.kiro','.claude','.codex'].map(harness=>{
    const path=join(home,harness,'skills','tkt'),entry=stat(path);
    if(!entry)return {path,action:'absent'};
    const parent=realpathSync(dirname(path));
    if(!parent.startsWith(home+sep))return {path,action:'preserve',reason:'skill root points outside the selected home'};
    if(!entry.isSymbolicLink())return {path,action:'preserve',reason:'not a legacy symlink; review manually'};
    const target=canonical(resolve(dirname(path),readlinkSync(path)));
    return target===source?{path,target,action:'remove-link'}:{path,target,action:'preserve',reason:'link does not target the supplied old source'};
  });
  const blocked=rows.some(row=>row.action==='preserve'),report={schema:1,status:blocked?'fail':'pass',mode:execute?'execute':'dry-run',source,home,rows,removed:0};
  // Preflight every root before changing any link; foreign content needs human review.
  if(execute&&!blocked)for(const row of rows)if(row.action==='remove-link'){unlinkSync(row.path);report.removed++;}
  return report;
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  try {
    const args=process.argv.slice(2),options={};
    while(args.length) {
      const key=args.shift();
      if(key==='--execute')options.execute=true;
      else if(['--source','--home'].includes(key)&&args.length)options[key.slice(2)]=args.shift();
      else throw Error(`Unknown or incomplete option: ${key}`);
    }
    const report=migrateLegacySkills(options);console.log(JSON.stringify(report,null,2));process.exitCode=report.status==='pass'?0:1;
  }catch(error){console.error(JSON.stringify({status:'error',errors:[error.message]}));process.exitCode=2;}
}
