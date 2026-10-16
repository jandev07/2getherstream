import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {fileURLToPath,pathToFileURL} from 'node:url';
const appRoot=fileURLToPath(new URL('../apps/windows/',import.meta.url));
const require=createRequire(new URL('../apps/windows/package.json',import.meta.url));
const {createServer}=await import(pathToFileURL(require.resolve('vite')).href);
// Optional external test runtime, not an application dependency. Existing browser
// payloads can be selected without installing or opening a user browser profile.
const playwright=process.env.TOGETHER_PLAYWRIGHT_MODULE;
const {chromium}=await import(playwright?pathToFileURL(playwright).href:'playwright');
const bridge=`export async function invoke(command,args){
 window.__surfaceMoves??=[];
 if(command==='video_bounds'){window.__surfaceMoves.push({...args});return;}
 if(command==='player_status'){window.__statusReads=(window.__statusReads??0)+1;return {initialized:true,state:'Paused',position_ms:0,duration_ms:7500000,buffering:false,error:window.__surfaceError??null,generation:1,drift_ms:0,rate:1};}
 if(command==='session_snapshot')return {generation:1,media:{provider:'fixture'},ended:false,prepare:null,participants:{pc:{connected:true,buffering:false,rtt_ms:0},phone:{connected:false,buffering:false,rtt_ms:0}},fault:null};
 if(command==='pairing_connection')return {emulator_address:'http://127.0.0.1:47820',phone_address:null,tunnel:{mode:'stopped'}};
 if(command==='android_download_info')return {url:'https://example.invalid',qr_svg:''};
 if(command==='startup_proof_enabled'||command==='private_alpha_account_links')return false;
 return null;
}`;
const server=await createServer({root:appRoot,server:{host:'127.0.0.1',port:0,strictPort:true},plugins:[{
 name:'isolated-surface-layout-test',enforce:'pre',
 resolveId(id){if(id==='@tauri-apps/api/core')return '\0surface-layout-bridge'},
 load(id){if(id==='\0surface-layout-bridge')return bridge},
}]});
let browser;const observations=[];
try{
 await server.listen();const address=server.httpServer.address();assert.ok(address && typeof address==='object');
 browser=await chromium.launch({headless:true,...(process.env.TOGETHER_TEST_BROWSER?{executablePath:process.env.TOGETHER_TEST_BROWSER}:{})});
 const page=await browser.newPage({viewport:{width:1100,height:850}});
 await page.goto('http://127.0.0.1:'+address.port);await page.getByRole('button',{name:'Watch',exact:true}).click();
 await page.waitForFunction(()=>window.__surfaceMoves?.at(-1)?.width>0);
 const observe=label=>page.evaluate(label=>{const r=document.querySelector('.video').getBoundingClientRect();return {label,dom:{x:r.x,y:r.y,width:r.width,height:r.height},submitted:window.__surfaceMoves.at(-1),calls:window.__surfaceMoves.length}},label);
 observations.push(await observe('initial'));
 for(const [label,message] of [['notice inserted','Synthetic native failure for layout test'],['notice wraps','Synthetic native failure for layout test. '.repeat(12)],['notice removed',null]]){
  await page.evaluate(message=>{window.__surfaceError=message},message);
  if(message)await page.getByRole('alert').filter({hasText:message}).waitFor();
  else await page.getByRole('alert').waitFor({state:'hidden'});
  try{await page.waitForFunction(()=>{const r=document.querySelector('.video').getBoundingClientRect();const last=window.__surfaceMoves.at(-1);return last && Math.abs(r.y-last.y)<1 && Math.abs(r.height-last.height)<1},null,{timeout:3000});}
  finally{observations.push(await observe(label));}
 }
 assert.ok(observations[1].dom.y>observations[0].dom.y,'notice did not move actual layout');
 assert.ok(observations[2].dom.y>observations[1].dom.y,'wrapped notice did not move actual layout');
 assert.equal(observations[3].dom.y,observations[0].dom.y);
 assert.ok(observations.every(row=>row.dom.width===observations[0].dom.width && row.dom.height===observations[0].dom.height),'test unexpectedly resized the video');
 const reads=await page.evaluate(()=>window.__statusReads);
 await page.waitForFunction(before=>window.__statusReads>=before+6,reads,{timeout:4000});
 const idle=await observe('six unchanged status refreshes');
 assert.equal(idle.calls,observations.at(-1).calls,'unchanged status refreshes submitted extra geometry');
 observations.push(idle);
 console.log('PASS rendered notice insertion/wrapping/removal align; six unchanged status refreshes add no geometry');
}finally{
 console.log(JSON.stringify({observations,scope:'Rendered App.svelte with stub IPC/status. No Tauri, Win32, libmpv, phone or timing acceptance.'}));
 await browser?.close();await server.close();
}
