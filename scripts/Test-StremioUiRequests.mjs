import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import { createRequire } from 'node:module';
const require = createRequire(new URL('../apps/windows/package.json', import.meta.url));
const ts = require('typescript');
const file = new URL('../apps/windows/src/App.svelte', import.meta.url);
const component = fs.readFileSync(file, 'utf8');
const script = component.match(/<script lang="ts">([\s\S]*?)<\/script>/)[1]
  .replace(/^import .*;\r?\n/gm, '');
const code = ts.transpileModule(script, { compilerOptions: { target: ts.ScriptTarget.ES2022 } }).outputText;
// Exercise the actual component request handlers with deferred IPC. This does
// not render Svelte/DOM or start Tauri, native players or provider requests.
function harness(series = false) {
  const calls = [];
  const context = vm.createContext({ $state: value => value, onMount: () => {},
    invoke: (command, args) => new Promise((resolve, reject) => calls.push({ command, args, resolve, reject })) });
  vm.runInContext(code, context);
  const browser = revision => ({ connected: true, revision, catalogs: [], episode_mode: false,
    titles: [{ id: 'test-title', kind: series ? 'series' : 'movie', name: 'Current title' }] });
  vm.runInContext(`browser=${JSON.stringify(browser(1))}`, context);
  return { calls, browser, run: expression => vm.runInContext(expression, context),
    state: () => JSON.parse(vm.runInContext('JSON.stringify({browser,options,stremioBusy,stremioCancelable,torboxBusy,selectedTitle,page,copiedLink,error,prepared:[...prepared],preparingMessage})', context)) };
}
const sources = revision => ({ revision, candidates: 1, unavailable_providers: 0, cache_available: true,
  ready: [{ index: 0, filename: 'movie.mp4', filesize: 4096 }], cached: [] });
async function cancel(h) {
  const pending = h.run('cancelStremioLookup()');
  const call = h.calls.at(-1); assert.equal(call.command, 'stremio_cancel_lookup');
  call.resolve(h.browser(2)); await pending;
}
{
  const h = harness(); const old = h.run('movieOptions(0)');
  await cancel(h); h.calls[0].resolve(sources(1)); await old;
  assert.equal(h.state().browser.revision, 2); assert.equal(h.state().options, null);
  assert.equal(h.state().stremioBusy, false);
}
{
  const h = harness(); const old = h.run('movieOptions(0)'); await cancel(h);
  const current = h.run('movieOptions(0)'); h.calls[0].reject(new Error('obsolete lookup'));
  await old; assert.equal(h.state().stremioBusy, true); assert.equal(h.state().error, '');
  h.calls[2].resolve(sources(3)); await current;
  assert.equal(h.state().options.revision, 3); assert.equal(h.state().stremioBusy, false);
}
{
  const h = harness(); const old = h.run('movieOptions(0)'); h.calls[0].reject(new Error('lookup failed'));
  // Allow the catch handler to start its recovery browser query.
  await new Promise(setImmediate);
  assert.equal(h.calls[1].command, 'stremio_browser'); await cancel(h);
  h.calls[1].resolve(h.browser(99)); await old;
  assert.equal(h.state().browser.revision, 2); assert.equal(h.state().error, '');
}
{
  const h = harness(true); const old = h.run('chooseTitle(0)');
  assert.equal(h.calls[0].command, 'stremio_episodes'); await cancel(h);
  h.calls[0].resolve({ ...h.browser(99), episode_mode: true }); await old;
  assert.equal(h.state().browser.revision, 2); assert.equal(h.state().browser.episode_mode, false);
}
{
  const h = harness(); const old = h.run('movieOptions(0)');
  const cancelling = h.run('cancelStremioLookup()');
  h.calls[1].reject(new Error('temporary IPC failure')); await cancelling;
  assert.equal(h.state().stremioCancelable, true, 'failed cancellation must remain retryable');
  await cancel(h); h.calls[0].reject(new Error('obsolete lookup')); await old;
  assert.equal(h.state().browser.revision, 2); assert.equal(h.state().error, '');
}
{
  const h = harness(); const old = h.run('browseStremio()');
  assert.equal(h.calls[0].command, 'stremio_browse'); await cancel(h);
  h.calls[0].resolve({ ...h.browser(99), titles: [{ id: 'old', kind: 'movie', name: 'Old result' }] }); await old;
  assert.equal(h.state().browser.revision, 2); assert.equal(h.state().browser.titles[0].name, 'Current title');
}
// Cross-provider source actions must not compete with transmitted preparation
// or publication. Exercise actual handlers, not a duplicate busy-policy model.
const sourceCalls=[
  ["watchStremio({index:0})",'stremio_select'],
  ["prepareStremio({index:0})",'stremio_prepare'],
  ["importStremio()",'stremio_copied_source'],
  ["selectTorbox({torrent_id:1,file_id:2,filename:'TorBox title'})",'torbox_select'],
  ["refreshTorbox()",'torbox_refresh'],
];
for(const [expression,command] of sourceCalls){
 for(const [other] of [...sourceCalls.slice(3),...sourceCalls.slice(0,3)]){
  const h=harness();h.run(`options=${JSON.stringify(sources(1))};movieName='Stremio title';copiedLink='synthetic-private-source'`);
  const first=h.run(expression);assert.equal(h.calls[0].command,command);
  const second=h.run(other);const attempted=h.calls.length;
  for(const call of h.calls)call.resolve({ready:false});
  await Promise.all([first,second]);
  assert.equal(attempted,1,`${command} admitted a competing source request`);
  assert.equal(h.state().stremioBusy,false);assert.equal(h.state().torboxBusy,false);
 }
}
{
 const h=harness();h.run(`options=${JSON.stringify(sources(1))};movieName='Stremio title';copiedLink='synthetic-private-source'`);
 const first=h.run('watchStremio({index:0})');const second=h.run('importStremio()');
 const retained=h.state().copiedLink;for(const call of h.calls)call.resolve({});await Promise.all([first,second]);
 assert.equal(retained,'synthetic-private-source','blocked import consumed source input');
 assert.equal(h.state().selectedTitle,'Stremio title');assert.equal(h.state().page,'watch');
}
for(const [expression,command] of sourceCalls){
 const h=harness();h.run(`options=${JSON.stringify(sources(1))};movieName='Stremio title';copiedLink='synthetic-private-source'`);
 const failed=h.run(expression);h.calls[0].reject(new Error('fixed synthetic source rejection'));await failed;
 assert.equal(h.state().stremioBusy,false);assert.equal(h.state().torboxBusy,false);
 if(command==='stremio_copied_source')h.run("copiedLink='synthetic-private-source'");
 const retry=h.run(expression);assert.equal(h.calls.length,2);assert.equal(h.calls[1].command,command);
 h.calls[1].resolve({ready:false});await retry;assert.equal(h.state().error,'');
}
for(const [expression] of sourceCalls){
 const h=harness();h.run(`options=${JSON.stringify(sources(1))};movieName='Stremio title'`);
 const first=h.run(expression);const read=h.run('loadTorbox()');const attempted=h.calls.length;
 for(const call of h.calls)call.resolve({ready:false});await Promise.all([first,read]);
 assert.equal(attempted,1,'inventory request changed busy ownership during source operation');
 assert.equal(h.state().stremioBusy,false);assert.equal(h.state().torboxBusy,false);
}
for(const [expression,command] of sourceCalls){
 for(const rejection of ['stale media resolution','TorBox selection replaced','Selection changed; choose the movie again','Movie selection replaced','server stopping']){
  const h=harness();h.run(`options=${JSON.stringify(sources(1))};movieName='New title';selectedTitle='Current movie';page='watch';copiedLink='synthetic-private-source'`);
  const pending=h.run(expression);h.calls[0].reject(rejection);await pending;
  const retry=command==='torbox_refresh'?'Refresh playback links again to retry.':command==='stremio_copied_source'?'Paste the source again to retry.':'Select the video again to retry.';
  const expected=rejection==='server stopping'?'Windows playback is closing. Reopen 2gether before choosing a video.':['Selection changed; choose the movie again','Movie selection replaced'].includes(rejection)?'Your movie selection changed. Refresh Stremio and choose the title again.':`Playback or your source selection changed while this request was running. ${retry}`;
  assert.equal(h.state().error,expected,`${command} must give recovery feedback`);
  assert.equal(h.state().selectedTitle,'Current movie');assert.equal(h.state().page,'watch');
  assert.equal(h.state().stremioBusy,false);assert.equal(h.state().torboxBusy,false);
 }
}
for(const [expression] of sourceCalls){
 const h=harness();h.run(`options=${JSON.stringify(sources(1))};copiedLink='synthetic-private-source'`);
 const pending=h.run(expression);h.calls[0].reject('Could not refresh ready TorBox videos');await pending;
 assert.equal(h.state().error,'Could not refresh ready TorBox videos','other fixed provider feedback must be preserved');
}
// Metadata must not replace choices or steal admission from a transmitted source action.
{
 const h=harness();h.run(`options=${JSON.stringify(sources(1))};movieName='Current title'`);
 const preparation=h.run('prepareStremio({index:0})');
 const replacement=h.run('connectStremio()');const attempted=h.calls.length;
 h.calls[0].resolve({ready:true,filename:'prepared.mp4',filesize:4096});await preparation;
 if(attempted>1){
  console.log('Controlled component overlap evidence: '+JSON.stringify(h.state()));
  h.calls[1].resolve(h.browser(2));
 }
 await replacement;
 assert.equal(attempted,1,'metadata was admitted while transmitted preparation still owned source choices');
 assert.equal(h.state().options.revision,1);assert.equal(h.state().options.ready.at(-1).filename,'prepared.mp4');
 assert.equal(h.state().preparingMessage,'Your video is ready. Select Watch to prepare both players.');
}
const metadataCalls=[
 ['connectStremio()',false],['browseStremio()',false],['movieOptions(0)',false],
 ['chooseTitle(0)',false],['backToSeries()',false],['chooseTitle(0)',true],
];
for(const [source,command] of sourceCalls){
 for(const [metadata,series] of metadataCalls){
  const h=harness(series);h.run(`options=${JSON.stringify(sources(1))};movieName='Original title';copiedLink='synthetic-private-source'`);
  const first=h.run(source);const second=h.run(metadata);const attempted=h.calls.length;
  const pending=h.state();for(const call of h.calls)call.resolve({ready:false});await Promise.all([first,second]);
  assert.equal(attempted,1,`${metadata} overlapped ${command}`);
  assert.equal(pending.options.revision,1,'blocked metadata cleared choices');
  assert.equal(pending.stremioBusy || pending.torboxBusy,true,'blocked metadata released source owner');
  assert.equal(h.state().stremioBusy,false);assert.equal(h.state().torboxBusy,false);
 }
}
for(const [source,series] of metadataCalls.slice(0,4)){
 for(const [next] of metadataCalls){
  const h=harness(series);const first=h.run(source);const second=h.run(next);const attempted=h.calls.length;
  for(const call of h.calls)call.resolve(call.command==='stremio_options'?sources(2):h.browser(2));
  await Promise.all([first,second]);assert.equal(attempted,1,`${next} overlapped pending metadata`);
  assert.equal(h.state().stremioBusy,false);
 }
}
for(const [next,series] of metadataCalls){
 const h=harness(series);const first=h.run('loadTorbox()');const second=h.run(next);const attempted=h.calls.length;
 for(const call of h.calls)call.resolve([]);await Promise.all([first,second]);assert.equal(attempted,1,'metadata stole inventory busy owner');
 assert.equal(h.state().torboxBusy,false);
}
for(const existing of [false,true]){
 const h=harness();const initial=sources(1);
 initial.ready=existing?[{index:0,filename:'old.mp4',filesize:1024},{index:1,filename:'other.mp4',filesize:2048}]:[];
 initial.cached=[{index:0,filename:'prepared.mp4',filesize:4096}];
 h.run(`options=${JSON.stringify(initial)}`);
 for(let attempt=0;attempt<2;attempt++){
  const request=h.run('prepareStremio({index:0})');h.calls.at(-1).resolve({ready:true,filename:'prepared.mp4',filesize:4096});await request;
 }
 const current=h.state().options;
 assert.equal(current.ready.filter(row=>row.index===0).length,1,'repeated preparation result duplicated a ready source');
 assert.equal(current.ready.find(row=>row.index===0).filename,'prepared.mp4');assert.equal(current.cached.length,0);
 if(existing)assert.equal(current.ready.find(row=>row.index===1).filename,'other.mp4');
}
// Returning to the title list is local navigation; it must not compete with
// a transmitted preparation/watch request or disturb backend choice ownership.
for (const busy of ['stremioBusy','torboxBusy',null]) {
 const h=harness();h.run(`options=${JSON.stringify(sources(7))};movieName='Chosen title';preparingMessage='Preparing';${busy ? busy+'=true' : ''}`);
 h.run('chooseAnotherTitle()');
 assert.equal(h.calls.length,0);
 assert.equal(h.state().browser.revision,1);
 if(busy){assert.equal(h.state().options.revision,7);assert.equal(h.state().preparingMessage,'Preparing');}
 else{assert.equal(h.state().options,null);assert.equal(h.state().preparingMessage,'');}
}
console.log('6 metadata ordering,36 source admission/recovery,30 source rejection feedback,61 metadata/source admission and2 repeated preparation checks plus3 title-navigation checks passed (deferred IPC; no rendered UI).');
