
wasm-pack build --target web --release -d pkg_miniapp --features wasm-miniapp

cd ./scripts

node fix_miniapp.js

cd ..

cp -r ./pkg_miniapp/* ~/MiniProjects/API-Demo/pkg

echo "done"