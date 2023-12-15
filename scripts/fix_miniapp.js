var fs = require("fs");  
const appName = "rust_nes"

var content = fs.readFileSync(`../pkg_miniapp/${appName}.js`, "utf-8");

// var encoder_js = fs.readFileSync("./encoding_utf8.min.js", "utf-8");

// 加入 encoder
// content = encoder_js + '\n' + content;
content = `import { TextEncoder, TextDecoder } from "./encoding";\n` + content; 

// 删除 init函数中的两个行
content = content.replace(`input = new URL('${appName}_bg.wasm', import.meta.url);`, '');

content = content.replace(`input = fetch(input);`, '');

fs.writeFileSync(`../pkg_miniapp/${appName}.fix.js`, content);

fs.unlinkSync(`../pkg_miniapp/${appName}.js`)
fs.renameSync(`../pkg_miniapp/${appName}.fix.js`,`../pkg_miniapp/${appName}.js`)
