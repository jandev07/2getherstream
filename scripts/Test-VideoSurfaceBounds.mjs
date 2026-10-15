import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import {createRequire} from 'node:module';
const require=createRequire(new URL('../apps/windows/package.json',import.meta.url));
const ts=require('typescript');
const component=fs.readFileSync(new URL('../apps/windows/src/App.svelte',import.meta.url),'utf8');
const script=component.match(/<script lang="ts">([\s\S]*?)<\/script>/)[1].replace(/^import .*;\r?\n/gm,'');
const code=ts.transpileModule(script,{compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText;
// Actual mounted component script with deferred IPC/frame/event stubs. This is
// not rendered DOM, Tauri, Win32 positioning or a native player measurement.
function harness(rect={x:20,y:120,width:640,height:360}){
 let mount;let resize;let nextFrame=0;let disposed=false;
 const frames=new Map();const events=new Map();const calls=[];
 const context=vm.createContext({$state:x=>x,onMount:fn=>{mount=fn},
  invoke:(command,args)=>new Promise((resolve,reject)=>calls.push({command,args,resolve,reject})),
  surfaceMock:{getBoundingClientRect:()=>({...rect})},
  requestAnimationFrame:fn=>{const id=nextFrame++;frames.set(id,fn);return id},
  cancelAnimationFrame:id=>frames.delete(id),
  ResizeObserver:class{constructor(fn){resize=fn}observe(){}disconnect(){disposed=true}},
  window:{addEventListener:(name,fn)=>events.set(name,fn),removeEventListener:name=>events.delete(name)},
  document:{querySelector:()=>null,querySelectorAll:()=>[]},setInterval:()=>1,clearInterval:()=>{}});
 vm.runInContext(code,context);vm.runInContext('surface=surfaceMock',context);
 const cleanup=mount();
 return {calls,frames,events,cleanup,rect:r=>{rect=r},resize:()=>resize(),
  moves:()=>calls.filter(c=>c.command==='video_bounds'),
  error:()=>vm.runInContext('error',context),disposed:()=>disposed,
  frame:async()=>{const pending=[...frames.values()];frames.clear();for(const fn of pending)fn();await new Promise(setImmediate)}};
}
const flush=()=>new Promise(setImmediate);
const cases=[
 ['negative viewport coordinates stay negative',async()=>{
  const h=harness({x:-15,y:-120,width:640,height:360});await h.frame();
  assert.equal(h.moves().length,1);assert.equal(h.moves()[0].args.x,-15);assert.equal(h.moves()[0].args.y,-120);
  assert.equal(h.moves()[0].args.height,360);
 }],
 ['events in one frame send only latest geometry',async()=>{
  const h=harness();for(let i=0;i<100;i++){h.rect({x:0,y:i,width:640,height:360});h.resize()}
  await h.frame();assert.equal(h.moves().length,1);assert.equal(h.moves()[0].args.y,99);
 }],
 ['held IPC coalesces one latest followup',async()=>{
  const h=harness();await h.frame();for(let i=0;i<100;i++){h.rect({x:0,y:-i,width:640,height:360});h.events.get('scroll')()}
  await h.frame();assert.equal(h.moves().length,1);
  h.moves()[0].resolve();await flush();await h.frame();
  assert.equal(h.moves().length,2);assert.equal(h.moves()[1].args.y,-99);
 }],
 ['hidden surface zero bounds replace pending geometry',async()=>{
  const h=harness();await h.frame();h.rect({x:0,y:0,width:0,height:0});h.resize();
  h.moves()[0].resolve();await flush();await h.frame();
  assert.equal(h.moves().length,2);assert.equal(h.moves()[1].args.width,0);assert.equal(h.moves()[1].args.height,0);
 }],
 ['failed current request releases ownership for resize retry',async()=>{
  const h=harness();await h.frame();h.moves()[0].reject('surface unavailable');await flush();
  assert.match(h.error(),/surface unavailable/);h.resize();await h.frame();assert.equal(h.moves().length,2);
 }],
 ['obsolete failure does not hide pending latest geometry',async()=>{
  const h=harness();await h.frame();h.rect({x:0,y:50,width:320,height:180});h.resize();
  h.moves()[0].reject('old move failed');await flush();await h.frame();
  assert.equal(h.error(),'');assert.equal(h.moves().length,2);assert.equal(h.moves()[1].args.width,320);
 }],
 ['cleanup cancels frame and removes event owners',async()=>{
  const h=harness();h.cleanup();await h.frame();assert.equal(h.moves().length,0);
  assert.equal(h.events.size,0);assert.equal(h.disposed(),true);
 }],
 ['cleanup prevents late error or followup submission',async()=>{
  const h=harness();await h.frame();h.resize();h.cleanup();h.moves()[0].reject('late move failed');
  await flush();await h.frame();assert.equal(h.error(),'');assert.equal(h.moves().length,1);
 }],
 ['completion-turn resize is retained',async()=>{
  const h=harness();await h.frame();h.moves()[0].resolve();await flush();
  h.rect({x:0,y:77,width:800,height:450});h.resize();await h.frame();
  assert.equal(h.moves().length,2);assert.equal(h.moves()[1].args.y,77);
 }],
];
let failed=0;
for(const [name,run] of cases){try{await run();console.log(`PASS ${name}`)}catch(error){failed++;console.error(`FAIL ${name}: ${error.message}`)}}
console.log(JSON.stringify({checks:cases.length,failed,rendered_ui_verified:false}));
if(failed)process.exitCode=1;
