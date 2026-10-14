import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import {createRequire} from 'node:module';
const require=createRequire(new URL('../apps/windows/package.json',import.meta.url));
const ts=require('typescript');
const component=fs.readFileSync(new URL('../apps/windows/src/App.svelte',import.meta.url),'utf8');
const script=component.match(/<script lang="ts">([\s\S]*?)<\/script>/)[1].replace(/^import .*;\r?\n/gm,'');
const code=ts.transpileModule(script,{compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText;
// Evaluate actual component button expressions and script, without a DOM/Tauri player.
function harness(player,media=true,deferred=false){
 const calls=[];
 const context=vm.createContext({$state:value=>value,onMount:()=>{},invoke:(command,args)=>new Promise((resolve,reject)=>{calls.push({command,args,resolve,reject});if(!deferred)resolve()})});
 vm.runInContext(code,context);
 vm.runInContext(`Object.assign(player,${JSON.stringify(player)});session=${JSON.stringify({media:media?{provider:'fixture'}:null})}`,context);
 const buttons=['−10 seconds','+10 seconds'].map(label=>{
  const markup=component.split('\n').find(line=>line.includes(`>${label}</button>`));
  return {disabled:()=>vm.runInContext(markup.match(/disabled=\{(.+?)\} onclick/)[1],context),
   click:()=>vm.runInContext(`(${markup.match(/onclick=\{(.+)\}>/)[1]})()`,context)};
 });
 return {calls,buttons,run:expression=>vm.runInContext(expression,context),error:()=>vm.runInContext('error',context)};
}
let checks=0;
for(const duration of [1,100,499,500,501,1000,30000]){
 const h=harness({initialized:true,duration_ms:duration,position_ms:0});
 assert.equal(h.buttons[1].disabled(),false);
 await h.buttons[1].click();
 assert.equal(h.calls[0].args.positionMs,Math.max(0,Math.min(duration-500,10000)),`forward target for ${duration}ms media`);
 assert.equal(h.calls[0].command,'player_intent');assert.equal(h.calls[0].args.intent,'seek');checks++;
}
for(const [position,delta,expected] of [[15000,-10000,5000],[15000,10000,25000],[2000,-10000,0],[29500,10000,29500],[45000,-10000,29500]]){
 const h=harness({initialized:true,duration_ms:30000,position_ms:position});
 await h.buttons[delta<0?0:1].click();assert.equal(h.calls[0].args.positionMs,expected);checks++;
}
for(const [player,media] of [[{initialized:false,duration_ms:30000,position_ms:2000},true],[{initialized:true,duration_ms:0,position_ms:0},true],[{initialized:true,duration_ms:30000,position_ms:2000},false]]){
 const h=harness(player,media);
 for(const button of h.buttons){assert.equal(button.disabled(),true);await button.click()}
 assert.equal(h.calls.length,0,'unavailable seek handler submitted an intention');checks++;
}
for(const olderFails of [true,false]){
 const h=harness({initialized:true,duration_ms:30000,position_ms:15000},true,true);
 const old=h.buttons[1].click();const latest=h.run("intent('pause')");
 assert.equal(h.calls.length,2,'latest pause must remain usable while seek reply is delayed');
 if(olderFails)h.calls[1].resolve();else h.calls[1].reject(new Error('latest pause failed'));
 await latest;
 const currentNotice=h.error();
 if(olderFails)h.calls[0].reject(new Error('obsolete seek failed'));else h.calls[0].resolve();
 await old;assert.equal(h.error(),currentNotice,'obsolete seek completion overwrote latest control feedback');checks++;
}
{
 const h=harness({initialized:true,duration_ms:30000,position_ms:15000},true,true);
 const failed=h.buttons[0].click();h.calls[0].reject(new Error('current seek failed'));await failed;
 assert.match(h.error(),/current seek failed/);
 const retry=h.buttons[0].click();h.calls[1].resolve();await retry;assert.equal(h.error(),'');checks++;
}
console.log(`${checks} actual seek button/script checks passed (no DOM/Tauri/native runtime)`);
