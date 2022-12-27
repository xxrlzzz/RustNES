const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const webpack = require("webpack");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
  entry: {
    index: "./index.js",
    client: "./client.js",
  },
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "[name]-[hash].js",
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: "index.html",
      filename: "index.html",
      chunks: ["index"],
    }),
    new HtmlWebpackPlugin({
      template: "client.html",
      filename: "client.html",
      chunks: ["client"],
    }),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, ".."),
      outDir: path.resolve(__dirname, "pkg"),
      forceMode: "production",
    }),
    // Have this example work in Edge which doesn't ship `TextEncoder` or
    // `TextDecoder` at this time.
    new webpack.ProvidePlugin({
      TextDecoder: ["text-encoding", "TextDecoder"],
      TextEncoder: ["text-encoding", "TextEncoder"],
    }),
  ],
  mode: "development",
  experiments: {
    asyncWebAssembly: true,
  },
  module: {
    rules: [
      {
        test: /\.nes$/,
        type: "asset/resource",
      },
    ],
  },
};
