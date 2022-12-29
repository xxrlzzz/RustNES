// from tetanes.

import * as wasm from "./pkg";
class State {
  constructor() {
    this.sample_rate = 44100;
    this.buffer_size = 1024;
    this.nes = null;
    this.animation_id = null;
    this.empty_buffers = [];
    this.audio_ctx = null;
    this.gain_node = null;
    this.next_start_time = 0;
    this.setup_audio();
  }

  load_rom(rom) {
    this.nes = wasm.WebNes.new(rom, "canvas", this.sample_rate);
    this.run();
  }

  setup_audio() {
    const AudioContext = window.AudioContext || window.webkitAudioContext;
    if (!AudioContext) {
      console.error("Browser does not support audio");
      return;
    }
    this.audio_ctx = new AudioContext();
    this.gain_node = this.audio_ctx.createGain();
    this.gain_node.gain.setValueAtTime(1, 0);
  }

  run() {
    this.animation_id = requestAnimationFrame(this.run.bind(this));
    this.nes.do_frame();
    this.queue_audio();
  }

  get_audio_buffer() {
    if (!this.audio_ctx) {
      throw new Error("AudioContext not created");
    }

    if (this.empty_buffers.length) {
      return this.empty_buffers.pop();
    } else {
      return this.audio_ctx.createBuffer(1, this.buffer_size, this.sample_rate);
    }
  }

  queue_audio() {
    if (!this.audio_ctx || !this.gain_node) {
      throw new Error("Audio not set up correctly");
    }

    this.gain_node.gain.setValueAtTime(1, this.audio_ctx.currentTime);

    const audioBuffer = this.get_audio_buffer();
    this.nes.audio_callback(this.buffer_size, audioBuffer.getChannelData(0));
    const source = this.audio_ctx.createBufferSource();
    source.buffer = audioBuffer;
    source.connect(this.gain_node).connect(this.audio_ctx.destination);
    source.onended = () => {
      this.empty_buffers.push(audioBuffer);
    };
    const latency = 0.032;
    const audio_ctxTime = this.audio_ctx.currentTime + latency;
    const start = Math.max(this.next_start_time, audio_ctxTime);
    source.start(start);
    this.next_start_time = start + this.buffer_size / this.sample_rate;
  }
  // ...
}

export default State;
