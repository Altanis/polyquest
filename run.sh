cd client
wasm-pack build --target=web
mv pkg/client_bg.wasm ../assets/client_bg.wasm
mv pkg/client_bg.wasm.d.ts ../assets/client_bg.wasm.d.ts
mv pkg/client.d.ts ../assets/client.d.ts
mv pkg/client.js ../assets/client.js
cd ..
cd assets
npx serve --no-clipboard