import { readFileSync, writeFileSync, mkdtempSync, mkdirSync, rmSync, existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve, dirname, basename } from 'node:path';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';

const source = readFileSync('src/telemetry.rs', 'utf8');
assert.equal(createHash('sha256').update(source).digest('hex'), '5f1d7c0e53de350f7f99fef86c327261962cd7560056d7a0a222902f32c431d9', 'Re-audit the extracted reader and private boundary before probing changed source');
const check = source.slice(source.indexOf('pub fn check_consent()'), source.indexOf('/// Path to the consent config file.'));
const read = source.slice(source.indexOf('fn read_consent_file('), source.indexOf('pub(crate) fn consent_plan('));
assert(check.startsWith('pub fn check_consent()'));
assert(read.startsWith('fn read_consent_file('));
const tempRoot = resolve(tmpdir());
const root = mkdtempSync(join(tempRoot, 'tkt-consent-147-'));
const canonical = join(root, 'canonical');
const alternate = join(root, 'alternate');
mkdirSync(canonical); mkdirSync(alternate);
const file = join(canonical, 'consent.toml');
const enabled = '[telemetry]\nenabled = true\nconsented_at = "2026-11-04"\nversion = 1\n';
const executable = join(root, process.platform === 'win32' ? 'reader.exe' : 'reader');
const rust = `use std::path::{Path,PathBuf};
#[derive(Debug,PartialEq)] enum Consent { Enabled,Disabled }
#[derive(Debug,PartialEq)] enum ConsentReason { DoNotTrack,EnvVar,CiDetected,ConfigFile,Default }
fn consent_file_path()->Option<PathBuf>{
 let p=PathBuf::from(std::env::var_os("TKT_PROBE_CONFIG").expect("private path required"));
 assert!(p.starts_with(${JSON.stringify(root)})); Some(p.join("consent.toml"))
}
${check}
${read}
fn main(){
 let _=consent_file_path();
 if std::env::var("TKT_PROBE_DNT").as_deref()==Ok("absent"){std::env::remove_var("DO_NOT_TRACK");}
 let (state,reason)=check_consent(); println!("{:?} {:?}",state,reason);
}
`;
const results=[];
try {
 writeFileSync(join(root,'reader.rs'),rust);
 execFileSync('rustc',['--edition=2021',join(root,'reader.rs'),'-o',executable],{env:{...process.env,DO_NOT_TRACK:'1'},stdio:'pipe'});
 function run(name,expected,overrides={},config=canonical){
  const env={...process.env,DO_NOT_TRACK:'1',TKT_PROBE_DNT:'absent',TKT_PROBE_CONFIG:config};
  delete env.TKT_TELEMETRY; delete env.CI;
  Object.assign(env,overrides);
  const before=readFileSync(file);
  const selected=join(config,'consent.toml');
  const selectedBefore=existsSync(selected)?readFileSync(selected):null;
  const actual=execFileSync(executable,[],{env,encoding:'utf8'}).trim();
  assert.equal(actual,expected,name);
  assert.deepEqual(readFileSync(file),before,`${name}: consent bytes changed`);
  assert.deepEqual(existsSync(selected)?readFileSync(selected):null,selectedBefore,`${name}: selected config path changed`);
  results.push({name,actual,unchanged:true});
 }
 writeFileSync(file,enabled);
 for(let i=1;i<=5;i++)run(`fresh-process-${i}`,'Enabled ConfigFile');
 run('DNT override','Disabled DoNotTrack',{TKT_PROBE_DNT:'present'});
 run('tool opt-out','Disabled EnvVar',{TKT_TELEMETRY:'off'});
 run('CI=true','Disabled CiDetected',{CI:'true'});
 run('CI empty still present','Disabled CiDetected',{CI:''});
 run('explicit on precedes CI','Enabled EnvVar',{TKT_TELEMETRY:'on',CI:'true'});
 run('DNT precedes explicit on','Disabled DoNotTrack',{TKT_PROBE_DNT:'present',TKT_TELEMETRY:'on'});
 run('alternate config root','Disabled Default',{},alternate);
 run('original root still enabled','Enabled ConfigFile');
 writeFileSync(file,'[telemetry]\nenabled = false\n');
 assert.throws(()=>run('injected reset','Enabled ConfigFile'),/injected reset/);
 results.push({name:'injected persistent reset',oracle:'rejected as expected'});
 writeFileSync(file,enabled);
 run('restored private fixture','Enabled ConfigFile');
 const report={platform:process.platform,source:'src/telemetry.rs',sha256:createHash('sha256').update(source).digest('hex'),scope:'Exact current consent reader/precedence source in fresh processes; platform path resolver substituted with bounded private roots. Not the whole CLI or a historical reporter sequence. No product code or real user consent changed.',results};
 console.log(JSON.stringify(report,null,2));
} finally {
 assert.equal(dirname(resolve(root)),tempRoot);
 assert(basename(root).startsWith('tkt-consent-147-'));
 rmSync(root,{recursive:true,force:true});
}
