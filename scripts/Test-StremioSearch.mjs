import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {fileURLToPath,pathToFileURL} from 'node:url';
const appRoot=fileURLToPath(new URL('../apps/windows/',import.meta.url));
const require=createRequire(new URL('../apps/windows/package.json',import.meta.url));
const {createServer}=await import(pathToFileURL(require.resolve('vite')).href);
const modulePath=process.env.TOGETHER_PLAYWRIGHT_MODULE;
const {chromium}=await import(modulePath?pathToFileURL(modulePath).href:'playwright');
const bridge=`const browser=revision=>({connected:true,revision,catalogs:[{name:'Synthetic search',kind:'movie',searchable:true,search_required:true},{name:'Synthetic top movies',kind:'movie',searchable:false,search_required:false},{name:'Synthetic discover movies',kind:'movie',searchable:true,search_required:false}],titles:[],episode_mode:false});
export async function invoke(command,args){
 if(command==='stremio_connect')return browser(1);
 if(command==='stremio_browse'){window.__queries??=[];window.__queries.push({search:args.search,index:args.index});return browser(window.__queries.length+1);}
 if(command==='player_status')return {initialized:true,state:'Idle',position_ms:0,duration_ms:0,buffering:false,error:null,generation:0,drift_ms:0,rate:1};
 if(command==='session_snapshot')return {generation:0,media:null,ended:false,prepare:null,participants:{pc:{connected:true},phone:{connected:false}},fault:null};
 if(command==='pairing_connection')return {emulator_address:'http://127.0.0.1:47820',phone_address:null,tunnel:{mode:'stopped'}};
 if(command==='android_download_info')return {url:'https://example.invalid',qr_svg:''};
 if(command==='startup_proof_enabled'||command==='private_alpha_account_links')return false;
 return null;
}`;
const server=await createServer({root:appRoot,server:{host:'127.0.0.1',port:0,strictPort:true},plugins:[{
 name:'isolated-stremio-search-test',enforce:'pre',
 resolveId(id){if(id==='@tauri-apps/api/core')return '\0stremio-search-bridge'},
 load(id){if(id==='\0stremio-search-bridge')return bridge},
}]});
let browser;const observations=[];
try{
 await server.listen();const address=server.httpServer.address();assert.ok(address && typeof address==='object');
 browser=await chromium.launch({headless:true,...(process.env.TOGETHER_TEST_BROWSER?{executablePath:process.env.TOGETHER_TEST_BROWSER}:{})});
 const page=await browser.newPage();
 await page.goto('http://127.0.0.1:'+address.port);await page.getByRole('button',{name:'Browse Stremio',exact:true}).click();
 const search=page.getByRole('textbox',{name:'Find a movie'});const submit=page.getByRole('button',{name:'Show titles',exact:true});
 await search.fill('🎬');observations.push({case:'one supplementary character',disabled:await submit.isDisabled()});
 assert.equal(observations.at(-1).disabled,true,'One Unicode character must not satisfy the minimum of two');
 const query='映'.repeat(160);await search.fill(query);assert.equal(await search.inputValue(),query);
 assert.equal(await submit.isEnabled(),true);await submit.click();
 await page.waitForFunction(()=>window.__queries?.length===1);
 assert.equal(await page.evaluate(()=>window.__queries[0].search),query);
 observations.push({case:'160 BMP search characters',characters:[...query].length,submitted:true});
 const catalog=page.getByRole('combobox',{name:/Catalog/});await catalog.selectOption('1');
 assert.equal(await search.count(),0,'Search field should be absent for a catalog without search');
 await submit.click();await page.waitForFunction(()=>window.__queries?.length===2);
 const unfiltered=await page.evaluate(()=>window.__queries[1]);
 observations.push({case:'catalog without search',index:unfiltered.index,hasQuery:unfiltered.search!==null});
 assert.equal(unfiltered.index,1);assert.equal(unfiltered.search,null,'Hidden search must not be sent to another catalog');
 await catalog.selectOption('0');assert.equal(await search.inputValue(),query,'Switching back preserves the user query');
 await catalog.selectOption('2');
 assert.equal(await search.inputValue(),query,'Switching to optional search catalog preserves the user query');
 await search.fill('');
 assert.equal(await submit.isEnabled(),true,'Empty query should be admitted for optional search');
 await submit.click();
 await page.waitForFunction(()=>window.__queries?.length===3);
 const optionalEmpty=await page.evaluate(()=>window.__queries[2]);
 observations.push({case:'optional empty search',index:optionalEmpty.index,hasQuery:optionalEmpty.search!==null});
 assert.equal(optionalEmpty.index,2);assert.equal(optionalEmpty.search,null,'Empty optional search must dispatch null');
 await search.fill('🎬');
 observations.push({case:'optional one-character search',disabled:await submit.isDisabled()});
 assert.equal(observations.at(-1).disabled,true,'One Unicode character must be blocked for optional search');
 const optionalQuery='映画';await search.fill(optionalQuery);assert.equal(await search.inputValue(),optionalQuery);
 assert.equal(await submit.isEnabled(),true,'Two Unicode characters must be admitted for optional search');
 await submit.click();
 await page.waitForFunction(()=>window.__queries?.length===4);
 const optionalFiltered=await page.evaluate(()=>window.__queries[3]);
 observations.push({case:'optional valid search',index:optionalFiltered.index,submittedQuery:optionalFiltered.search});
 assert.equal(optionalFiltered.index,2);assert.equal(optionalFiltered.search,optionalQuery,'Valid optional search must dispatch user query');
 console.log('PASS Unicode search and catalog-specific query dispatch');
}finally{
 console.log(JSON.stringify({observations,scope:'Rendered App.svelte with synthetic IPC; no addon requests, native playback or phone/WAN acceptance.'}));
 await browser?.close();await server.close();
}
