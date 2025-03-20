cd client
wasm-pack build --target=web --out-dir ../assets
cd ../assets
npx serve --no-clipboard