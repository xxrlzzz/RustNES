// For more comments about what's going on here, check out the `hello_world`
// example.
import * as wasm from "./pkg";
// import * as wasm from "rust_nes";
import mario_url from "./assets/mario.nes";
console.log("1")
let engine = new RTCEngine();
engine.init();
let stream = document.getElementById("canvas").captureStream(30);
engine.initSender("ws://localhost:8099", stream);
console.log("engine", engine);

let channel = engine.createDataChannel("input");

channel.onopen = () => {
  console.log("data channel open");
};
engine.onDataChannelMessage = (event) => {
  let keys = event.data.split(",").map((x) => parseInt(x));
  wasm.update_remote_keyboard_state(keys);
  console.log("recv", keys);
};
channel.onclose = () => {
  console.log("data channel close");
};

wasm.wasm_main();
fetch(mario_url, {
  headers: {
    "Content-Type": "application/octet-stream",
  },
})
  .then((rsp) => {
    return rsp.arrayBuffer();
  })
  .then((array) => {
    let mario = new Uint8Array(array);
    wasm.start(mario, "canvas");
  });
