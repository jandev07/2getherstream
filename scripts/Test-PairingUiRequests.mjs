import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import {createRequire} from 'node:module';
const require=createRequire(new URL('../apps/windows/package.json',import.meta.url));
const ts=require('typescript');
const component=fs.readFileSync(new URL('../apps/windows/src/App.svelte',import.meta.url),'utf8');
const script=component.match(/<script lang="ts">([\s\S]*?)<\/script>/)[1].replace(/^import .*;\r?\n/gm,'');
const code=ts.transpileModule(script,{compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText;
// Actual component script, deferred IPC; no DOM, Tauri or public tunnel operation.
function harness(){
 const calls=[];
 const context=vm.createContext({$state:value=>value,onMount:()=>{},invoke:(command)=>new Promise((resolve,reject)=>calls.push({command,resolve,reject}))});
 vm.runInContext(code,context);vm.runInContext("pairingCode='OLD-CODE'",context);
 return {calls,run:e=>vm.runInContext(e,context),state:()=>JSON.parse(vm.runInContext('JSON.stringify({pairingCode,pairingBusy,connectionBusy,error,page,connection})',context))};
}
const connection={emulator_address:'synthetic-loopback',phone_address:null,tunnel:{mode:'stopped'}};
async function settle(h,pending){
 for(let index=0;index<8;index++){
  for(const call of h.calls)if(!call.settled){call.settled=true;call.resolve(call.command==='pairing_connection'?connection:call.command==='open_pairing'?'NEW-CODE':undefined)}
  await new Promise(setImmediate);
 }
 await pending;
}
const operations=['stopPhoneConnection()','pairPhone()','closePairing()','startPhoneConnection()'];
let checks=0;
for(const first of operations)for(const second of [...operations.filter(other=>other!==first),first]){
 const h=harness();const pending=h.run(first);const before=h.calls.length;
 const competing=h.run(second);const after=h.calls.length;
 await settle(h,Promise.all([pending,competing]));
 assert.equal(after,before,`${first} admitted ${second} while its request was pending`);
 assert.equal(h.state().pairingBusy,false);assert.equal(h.state().connectionBusy,false);checks++;
}
{
 const h=harness();const pending=h.run('closePairing()');
 assert.equal(h.state().pairingCode,'OLD-CODE','code disappeared before cancellation was confirmed');
 h.calls[0].reject(new Error('synthetic cancellation failure'));await pending;
 assert.equal(h.state().pairingCode,'OLD-CODE');assert.match(h.state().error,/cancel.*Try again/i);
 assert.equal(h.state().pairingBusy,false);
 const retry=h.run('closePairing()');h.calls[1].resolve();await retry;
 assert.equal(h.state().pairingCode,'');assert.equal(h.state().error,'');checks++;
}
{
 const h=harness();const pending=h.run('stopPhoneConnection()');
 h.calls[0].resolve();await new Promise(setImmediate);
 assert.equal(h.calls[1].command,'cancel_pairing','stopped connection must cancel code before display refresh');
 assert.equal(h.state().pairingCode,'');h.calls[1].resolve();await new Promise(setImmediate);
 assert.equal(h.calls[2].command,'pairing_connection');h.calls[2].reject(new Error('synthetic refresh failure'));await pending;
 assert.equal(h.state().pairingCode,'');assert.equal(h.state().connectionBusy,false);checks++;
}
for(const operation of operations){
 const h=harness();const pending=h.run(operation);h.calls[0].reject(new Error('synthetic first-stage failure'));await pending;
 assert.equal(h.state().pairingBusy,false);assert.equal(h.state().connectionBusy,false);assert.notEqual(h.state().error,'');
 const retry=h.run(operation);assert.equal(h.calls.length,2);await settle(h,retry);assert.equal(h.state().error,'');checks++;
}
{
 const h=harness();const waiting=Array.from({length:25},()=>h.run('refreshConnection()'));
 assert.equal(h.calls.length,1,'address refresh accumulated IPC while first query was blocked');
 h.calls[0].resolve({...connection,emulator_address:'OLD'});await new Promise(setImmediate);
 assert.equal(h.calls.length,2,'latest requested display needs one follow-up after held query');
 assert.equal(h.state().connection,null,'obsolete response briefly overwrote display');
 h.calls[1].resolve({...connection,emulator_address:'CURRENT'});await Promise.all(waiting);
 assert.equal(h.state().connection.emulator_address,'CURRENT');checks++;
}
{
 const h=harness();const first=h.run('refreshConnection()');const latest=h.run('refreshConnection()');
 h.calls[0].reject(new Error('obsolete read failure'));await new Promise(setImmediate);
 assert.equal(h.calls.length,2,'obsolete read failure discarded already-requested latest refresh');
 h.calls[1].resolve(connection);await Promise.all([first,latest]);assert.equal(h.state().connection.emulator_address,connection.emulator_address);checks++;
}
{
 const h=harness();const first=h.run('refreshConnection()');const latest=h.run('refreshConnection()');
 const results=Promise.allSettled([first,latest]);h.calls[0].resolve(connection);await new Promise(setImmediate);
 h.calls[1].reject(new Error('latest fixed read failure'));assert.ok((await results).every(result=>result.status==='rejected'));
 const retry=h.run('refreshConnection()');assert.equal(h.calls.length,3);h.calls[2].resolve(connection);await retry;checks++;
}
{
 const h=harness();const old=h.run('refreshConnection()');const start=h.run('startPhoneConnection()');
 assert.equal(h.calls[1].command,'start_temporary_connection');h.calls[1].resolve();await new Promise(setImmediate);
 assert.equal(h.calls.length,2,'completed start launched another read before earlier query finished');
 h.calls[0].resolve({...connection,phone_address:null});await new Promise(setImmediate);
 assert.equal(h.calls[2].command,'pairing_connection');
 h.calls[2].resolve({...connection,phone_address:'https://synthetic.invalid',tunnel:{mode:'temporary'}});
 await Promise.all([old,start]);assert.equal(h.state().connection.phone_address,'https://synthetic.invalid');assert.equal(h.state().connectionBusy,false);checks++;
}
// Exercise arrivals at multiple completion turns without assuming cross-realm
// promises and host microtasks resume in the same turn.
for(let turns=0;turns<=6;turns++){
 const h=harness();const first=h.run('refreshConnection()');h.calls[0].resolve(connection);
 for(let turn=0;turn<turns;turn++)await Promise.resolve();
 const second=h.run('refreshConnection()');await new Promise(setImmediate);
 assert.equal(h.calls.length,2,'completion-window refresh was lost');
 const third=h.run('refreshConnection()');assert.equal(h.calls.length,2,'older completion cleared current read ownership');
 await settle(h,Promise.all([first,second,third]));assert.equal(h.state().connection.emulator_address,connection.emulator_address);checks++;
}
console.log(`${checks} pairing/connection component request checks passed (no DOM/Tauri/tunnel)`);
