import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';

const html=fs.readFileSync(new URL('../docs/status.html',import.meta.url),'utf8');
const block=id=>JSON.parse(html.match(new RegExp('id="'+id+'">([\\s\\S]*?)</script>'))[1]);
const source=html.match(/<script>([\s\S]*?)<\/script>/g).at(-1).replace(/<\/?script>/g,'');

// This exercises the actual renderer/events in a small DOM model; it does not
// establish browser rendering, native radio behavior or OS clipboard access.
function page(clipboard, overrides={}) {
  const nodes=new Map(), all=[], events={};
  const document={activeElement:null};
  class Element {
    constructor(tag){this.tagName=tag.toUpperCase();this.attrs={};this.children=[];this.handlers={};this.value='';this.hidden=false;this.style={};this.dataset={};all.push(this);}
    setAttribute(k,v){this.attrs[k]=String(v);if(k==='id')nodes.set(v,this);if(k==='value')this.value=v;if(k==='hidden')this.hidden=true;}
    getAttribute(k){return this.attrs[k]??null;}
    get id(){return this.attrs.id;}
    set id(v){this.setAttribute('id',v);}
    get checked(){return !!this.isChecked;}
    set checked(v){if(v&&this.attrs.type==='radio')for(const n of all)if(n!==this&&n.attrs.type==='radio'&&n.attrs.name===this.attrs.name)n.isChecked=false;this.isChecked=v;}
    set textContent(v){this.text=String(v);this.children=[];}
    get textContent(){return (this.text||'')+this.children.map(n=>n.textContent).join('');}
    appendChild(n){n.parentNode=this;this.children.push(n);return n;}
    addEventListener(k,f){(this.handlers[k]??=[]).push(f);}
    fire(k,event={}){for(const f of this.handlers[k]||[])f(event);}
    querySelectorAll(){return [];}
    focus(){document.activeElement=this;}
    select(){this.selected=true;}
    scrollIntoView(){this.scrolled=true;}
  }
  document.createElement=t=>new Element(t);
  document.createTextNode=t=>{const n=new Element('#text');n.textContent=t;return n;};
  document.getElementById=id=>nodes.get(id)||null;
  for(const match of html.matchAll(/<([a-z][a-z0-9-]*)\b([^>]*\bid="([^"]+)"[^>]*)>/g))if(!nodes.has(match[3])){
    const n=new Element(match[1]);n.id=match[3];n.hidden=/\bhidden\b/.test(match[2]);n.parentNode=new Element('section');
  }
  const tablist=new Element('div');document.querySelector=()=>tablist;
  const cfg={...block('gm-config'),...overrides,ticketFlow:false};
  nodes.get('gm-config').textContent=JSON.stringify(cfg);
  nodes.get('gm-entries').textContent='[]';
  nodes.get('gm-tickets').textContent=JSON.stringify({tickets:[],frontier:[]});
  let hash='#decisions';
  const location={get hash(){return hash;},set hash(value){hash=value.startsWith('#')?value:'#'+value;}};
  const window={addEventListener:(k,f)=>events[k]=f};
  vm.runInNewContext(source,{document,window,location,navigator:{clipboard},Promise});
  return {nodes,all,cfg,location,events,document,window,tablist};
}
const settle=()=>new Promise(resolve=>setImmediate(resolve));
const drafts=()=>({decisions:block('gm-config').decisions.map(d=>({...d,status:'open'}))});

test('keyboard navigation includes the new tab and wraps through all four views',()=>{
  const p=page(null,drafts());
  p.tablist.fire('keydown',{key:'ArrowRight',preventDefault(){}});p.events.hashchange();
  assert.equal(p.location.hash,'#plan');assert.equal(p.document.activeElement,p.nodes.get('tab-plan'));
  p.tablist.fire('keydown',{key:'ArrowRight',preventDefault(){}});p.events.hashchange();
  assert.equal(p.location.hash,'#overview');assert.equal(p.nodes.get('overview').hidden,false);
  p.tablist.fire('keydown',{key:'ArrowLeft',preventDefault(){}});p.events.hashchange();
  assert.equal(p.location.hash,'#plan');
});

test('every recorded decision gate has curated choices and a recommended default',()=>{
  const p=page(null,drafts());
  for(const ticket of block('gm-tickets').tickets.filter(t=>t.gate?.kind==='decision'))
    assert.ok(p.cfg.decisions.some(d=>d.tickets?.includes(ticket.id)),ticket.id);
  assert.ok(p.cfg.decisions.some(d=>d.tickets?.includes('226')));
  for(const d of p.cfg.decisions.filter(d=>d.status==='open')){
    const radios=p.all.filter(n=>n.attrs.name==='decision-'+d.id);
    assert.equal(radios.length,d.options.length);
    assert.equal(radios.filter(n=>n.checked).length,1);
    assert.equal(radios.find(n=>n.checked).value,d.recommended);
    assert.ok(p.nodes.get('decision-notes-'+d.id));
    assert.ok(p.nodes.get('decision-'+d.id).textContent.includes(d.reason));
    assert.ok(p.all.some(n=>n.tagName==='LABEL'&&n.attrs.for==='decision-notes-'+d.id));
  }
});

test('overview links target decisions and answered outcomes remain available',()=>{
  const cfg=block('gm-config'), first=cfg.decisions[0], p=page(null,drafts());
  assert.ok(p.nodes.get('needs').children.some(n=>n.children.some(a=>a.attrs.href==='#decision-'+first.id)));
  p.location.hash='#decision-'+first.id;p.events.hashchange();
  assert.equal(p.nodes.get('decisions').hidden,false);
  assert.equal(p.nodes.get('decision-'+first.id).scrolled,true);
  const done=page(null,{decisions:[{...first,status:'answered',answer:'Confirmed in chat'}]});
  assert.equal(done.nodes.get('decision-export').hidden,true);
  assert.ok(done.all.some(n=>n.tagName==='DETAILS'&&n.textContent.includes('Confirmed in chat')));
  assert.ok(!done.nodes.get('decision-'+first.id));
});

test('changing a choice and notes survives tab switches and copies the complete message',async()=>{
  let copied;
  const p=page({writeText:async text=>{copied=text;}},drafts()), d=p.cfg.decisions[0];
  p.nodes.get('decision-'+d.id+'-'+d.options[1].id).checked=true;
  p.nodes.get('decision-notes-'+d.id).value='  Keep this condition\nand this alternative.  ';
  p.location.hash='#plan';p.events.hashchange();p.location.hash='#decisions';p.events.hashchange();
  assert.equal(p.nodes.get('decisions').hidden,false);
  p.nodes.get('copy-decisions').fire('click');
  assert.equal(p.nodes.get('copy-decisions').disabled,true);
  await settle();
  assert.match(copied,new RegExp('Choice: '+d.options[1].label.replace(/[.*+?^${}()|[\]\\]/g,'\\$&')));
  assert.ok(copied.includes('Notes: Keep this condition\nand this alternative.'));
  for(const decision of p.cfg.decisions)assert.ok(copied.includes(decision.question));
  assert.equal(p.nodes.get('decision-reply').value,copied);
  assert.match(p.nodes.get('decision-copy-status').textContent,/Copied/);
  assert.equal(p.nodes.get('copy-decisions').disabled,false);
  assert.ok(p.cfg.decisions.every(d=>d.status==='open'));
});

for(const [name,clipboard] of [['absent',null],['rejected',{writeText:async()=>{throw Error('denied');}}],['throws',{writeText:()=>{throw Error('denied');}}]])
  test('clipboard '+name+' leaves selectable answers',async()=>{
    const p=page(clipboard,drafts());p.nodes.get('copy-decisions').fire('click');await settle();
    const reply=p.nodes.get('decision-reply');
    assert.equal(reply.hidden,false);assert.equal(reply.selected,true);assert.equal(p.document.activeElement,reply);
    assert.match(p.nodes.get('decision-copy-status').textContent,/Clipboard unavailable/);
    assert.ok(reply.value.includes('Decisions for tkt'));
    assert.equal(p.nodes.get('copy-decisions').disabled,false);
  });

test('submitted owner answers remain in history without another pending form',()=>{
  const p=page(null);
  assert.equal(p.cfg.decisions.length,17);
  assert.ok(p.cfg.decisions.every(d=>d.status==='answered'&&d.options.some(o=>o.id===d.selectedOption&&o.label===d.answer)));
  assert.equal(p.nodes.get('decision-export').hidden,true);
  for(const d of p.cfg.decisions)assert.ok(p.all.some(n=>n.tagName==='DETAILS'&&n.textContent.includes(d.answer)));
});
