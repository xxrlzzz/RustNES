# project_path="~/MiniProjects/API-Demo/pkg"
project_path="/Users/xxrl/codebase/webim-alipay"

wasm-pack build --target web --release -d pkg_miniapp --features wasm-miniapp

cd ./scripts

node fix_miniapp.js

cd ..


cp -r ./pkg_miniapp/* $project_path/pkg

echo "done"