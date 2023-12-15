// For more comments about what's going on here, check out the `hello_world`
// example.
import * as wasm from "./pkg";
// import * as wasm from "rust_nes";
import mario_url from "./assets/mario.nes";
import State from "./state";

const canvasEle = document.getElementById("canvas");
const ctx = canvasEle.getContext("webgl2");

function initRtc(canvasEle, ws_addr) {
  let engine = new RTCEngine();
  engine.init();
  let stream = canvasEle.captureStream(30);
  engine.initSender(ws_addr, stream);
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
  return engine;
}

let engine = initRtc(canvasEle, "ws://localhost:8099");


let state = new State();

try {
  wasm.wasm_main();
} catch (error) {
  console.error(error);
}
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
    state.load_rom(mario, ctx);
  }).catch((err) => {
    console.error(err);
  }) ;
