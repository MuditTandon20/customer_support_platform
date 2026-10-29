import {defineConfig} from '@playwright/test';
export default defineConfig({testDir:'./browser',timeout:120000,workers:1,use:{baseURL:process.env.TEST_WEB_URL||'http://localhost:13000',headless:true,screenshot:'only-on-failure',trace:'retain-on-failure'},reporter:[['list'],['html',{open:'never'}]]});
