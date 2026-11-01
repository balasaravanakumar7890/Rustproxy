import { spawn } from 'child_process';
import http from 'http';
import fs from 'fs';
import path from 'path';

const SCREENSHOT_DIR = path.resolve('docs/screenshots');
if (!fs.existsSync(SCREENSHOT_DIR)) {
  fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
}

// 1. Upstream mock server
const mockServer = http.createServer((req, res) => {
  if (req.url === '/api/v1/auth/login') {
    res.writeHead(200, {
      'Content-Type': 'application/json',
      'Set-Cookie': 'session_id=s3cur3t0k3n; Path=/',
    });
    res.end(JSON.stringify({ status: 'success', token: 'jwt_defensive_test_token' }));
  } else if (req.url === '/api/v1/users') {
    res.writeHead(200, {
      'Content-Type': 'application/json',
    });
    res.end(JSON.stringify({ users: [{ id: 1, name: 'Alice', role: 'admin' }, { id: 2, name: 'Bob', role: 'analyst' }] }));
  } else {
    res.writeHead(200, {
      'Content-Type': 'text/html; charset=utf-8',
      'Server': 'VulnerableApp/1.0',
    });
    res.end('<!DOCTYPE html><html><body><h1>Internal Application Gateway</h1></body></html>');
  }
});

await new Promise((resolve) => mockServer.listen(9099, '127.0.0.1', resolve));
console.log('[*] Mock upstream listening on 127.0.0.1:9099');

// 2. Start RustProxy binary
const proxyProc = spawn('./target/release/rustproxy', [], {
  stdio: 'inherit',
});

// Wait 2 seconds for RustProxy to start
await new Promise((r) => setTimeout(r, 2000));

// 3. Send sample traffic through proxy (127.0.0.1:8080)
async function sendProxyRequest(path, method = 'GET', body = null) {
  return new Promise((resolve) => {
    const req = http.request(
      {
        host: '127.0.0.1',
        port: 8080,
        method: method,
        path: `http://127.0.0.1:9099${path}`,
        headers: {
          Host: '127.0.0.1:9099',
          'User-Agent': 'Mozilla/5.0 (DefensiveTesting/1.0)',
          ...(body ? { 'Content-Type': 'application/json', 'Content-Length': Buffer.byteLength(body) } : {}),
        },
      },
      (res) => {
        let data = '';
        res.on('data', (c) => (data += c));
        res.on('end', () => resolve(data));
      }
    );
    req.on('error', (e) => resolve(null));
    if (body) req.write(body);
    req.end();
  });
}

console.log('[*] Generating sample traffic...');
await sendProxyRequest('/');
await sendProxyRequest('/api/v1/auth/login', 'POST', JSON.stringify({ username: 'admin', password: 'testpassword' }));
await sendProxyRequest('/api/v1/users');
await sendProxyRequest('/static/styles.css');
await sendProxyRequest('/admin/panel');

// Create a Repeater tab via API
try {
  await fetch('http://127.0.0.1:8081/api/repeater/tabs', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      name: 'Auth Test',
      request: {
        id: 'repeater-demo-1',
        method: 'POST',
        uri: 'http://127.0.0.1:9099/api/v1/auth/login',
        version: 'HTTP/1.1',
        headers: [
          ['Host', '127.0.0.1:9099'],
          ['Content-Type', 'application/json'],
          ['User-Agent', 'RustProxy-Repeater/1.0'],
        ],
        host: '127.0.0.1',
        port: 9099,
        scheme: 'http',
        path: '/api/v1/auth/login',
        body_length: 37,
        body: '{"username":"admin","password":"secret"}',
      },
    }),
  });
} catch (e) {
  console.log('Failed to pre-create repeater tab:', e);
}

await new Promise((r) => setTimeout(r, 1000));

// 4. Start Chromium with remote debugging
console.log('[*] Launching Chromium headless...');
const chromeProc = spawn('chromium', [
  '--headless',
  '--disable-gpu',
  '--remote-debugging-port=9222',
  '--window-size=1400,900',
  '--hide-scrollbars',
  'http://127.0.0.1:8081',
]);

await new Promise((r) => setTimeout(r, 2500));

// Query target list from CDP
const targetsRes = await fetch('http://127.0.0.1:9222/json/list');
const targets = await targetsRes.json();
const target = targets.find((t) => t.type === 'page');

if (!target) {
  console.error('No page target found');
  process.exit(1);
}

console.log('[*] Connected to target WebSocket:', target.webSocketDebuggerUrl);
const ws = new WebSocket(target.webSocketDebuggerUrl);

await new Promise((resolve) => ws.onopen = resolve);

let msgId = 1;
const pending = new Map();

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  if (data.id && pending.has(data.id)) {
    const { resolve, reject } = pending.get(data.id);
    pending.delete(data.id);
    if (data.error) reject(data.error);
    else resolve(data.result);
  }
};

function send(method, params = {}) {
  const id = msgId++;
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    ws.send(JSON.stringify({ id, method, params }));
  });
}

await send('Page.enable');
await send('DOM.enable');
await send('Runtime.enable');

// Wait for SPA to render fully
await new Promise((r) => setTimeout(r, 2000));

async function captureTab(navIndex, fileName) {
  console.log(`[*] Capturing tab index ${navIndex} -> ${fileName}...`);
  // Click nav button
  await send('Runtime.evaluate', {
    expression: `(() => {
      const btns = document.querySelectorAll('header nav button');
      if (btns[${navIndex}]) {
        btns[${navIndex}].click();
        return true;
      }
      return false;
    })()`,
  });

  await new Promise((r) => setTimeout(r, 1200));

  const shot = await send('Page.captureScreenshot', { format: 'png' });
  const buffer = Buffer.from(shot.data, 'base64');
  fs.writeFileSync(path.join(SCREENSHOT_DIR, fileName), buffer);
  console.log(`[+] Saved ${fileName} (${buffer.length} bytes)`);
}

// Tabs in order:
// 0: Dashboard
// 1: Proxy
// 2: HTTP History
// 3: Repeater
// 4: Target
// 5: Scanner
// 6: Decoder
// 7: Comparer

await captureTab(0, 'dashboard.png');
await captureTab(1, 'proxy_intercept.png');
await captureTab(2, 'history.png');
await captureTab(3, 'repeater.png');
await captureTab(4, 'sitemap.png');
await captureTab(5, 'scanner.png');
await captureTab(6, 'decoder.png');
await captureTab(7, 'comparer.png');

console.log('[*] All screenshots captured successfully!');

ws.close();
chromeProc.kill('SIGKILL');
proxyProc.kill('SIGKILL');
mockServer.close();
process.exit(0);
