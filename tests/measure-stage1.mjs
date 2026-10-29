import {createRequire} from 'node:module';
import {pathToFileURL} from 'node:url';
import {resolve} from 'node:path';
import {writeFile} from 'node:fs/promises';
import {chromium} from '@playwright/test';
if(!process.env.LIGHTHOUSE_INSTALL_DIR)throw new Error('Set LIGHTHOUSE_INSTALL_DIR to a separate local Lighthouse tool installation');
const tool=createRequire(resolve(process.env.LIGHTHOUSE_INSTALL_DIR,'package.json'));
const {default:lighthouse}=await import(pathToFileURL(tool.resolve('lighthouse')).href);
const {launch}=await import(pathToFileURL(tool.resolve('chrome-launcher')).href);
const chrome=await launch({chromePath:chromium.executablePath(),chromeFlags:['--headless','--disable-dev-shm-usage']});
const url=process.env.TEST_WEB_URL||'http://localhost:13000';
const prefix=process.env.PERFORMANCE_PREFIX||'STAGE1';
try{
 const result=await lighthouse(url,{port:chrome.port,onlyCategories:['performance','accessibility'],output:'json',logLevel:'error',enableErrorReporting:false});
 const lhr=result.lhr;
 const report={environment:'Local Windows, production Next.js, Playwright Chromium, Lighthouse 12.6.1 mobile simulated throttling; not hosted',url,scores:Object.fromEntries(Object.entries(lhr.categories).map(([k,v])=>[k,v.score===null?null:Math.round(v.score*100)])),lcp_ms:lhr.audits['largest-contentful-paint'].numericValue,cls:lhr.audits['cumulative-layout-shift'].numericValue,tbt_ms:lhr.audits['total-blocking-time'].numericValue,inp:'Not measured by a navigation Lighthouse run; no field p75 claimed',bootup:lhr.audits['bootup-time']?.details?.items||[],mainthread:lhr.audits['mainthread-work-breakdown']?.details?.items||[],warnings:lhr.runWarnings||[],runtimeError:lhr.runtimeError||null};
 if(process.env.MEASUREMENT_ENV)report.environment=process.env.MEASUREMENT_ENV;
 await writeFile(`../docs/${prefix}_LIGHTHOUSE.json`,JSON.stringify(report,null,2));
 console.log(JSON.stringify(report,null,2));
}finally{await chrome.kill();}
const browser=await chromium.launch();const page=await browser.newPage({viewport:{width:390,height:844}});
await page.addInitScript(()=>{window.__asterEvents=[];window.__asterCls=0;new PerformanceObserver(list=>{for(const e of list.getEntries())if(e.interactionId)window.__asterEvents.push({name:e.name,duration:e.duration});}).observe({type:'event',buffered:true,durationThreshold:16});new PerformanceObserver(list=>{for(const e of list.getEntries())if(!e.hadRecentInput)window.__asterCls+=e.value;}).observe({type:'layout-shift',buffered:true});});
await page.goto(url+'/login');await page.getByLabel('Email address').fill('synthetic-measurement@example.test');await page.getByLabel('Password',{exact:true}).fill('Synthetic-measurement-password!');
await page.getByRole('link',{name:'Join your team'}).click();
const interactions=await page.evaluate(()=>({samples:window.__asterEvents,cls:window.__asterCls}));
await writeFile(`../docs/${prefix}_INTERACTIONS.json`,JSON.stringify({environment:'Single local synthetic mobile interaction sample; not field INP',...interactions},null,2));
await browser.close();
