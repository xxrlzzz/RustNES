import * as wasm from "./pkg";

let engine = new RTCEngine();
console.log("engine", engine);
engine.init();
engine.initReceiver(
  "ws://localhost:8099",
  document.getElementById("remoteVideo")
);

let channel = engine.createDataChannel("input");
let timer = null;

timer = setInterval(sendInput, 16);
channel.onopen = () => {
  console.log("data channel open");
};
channel.onclose = () => {
  console.log("data channel close");
  clearInterval(timer);
};
function sendInput() {
  // console.log("call", engine);
  let data = wasm.keyboard_status();
  if (channel.readyState !== "open") return;
  channel.send(data);
}
wasm.wasm_main();
