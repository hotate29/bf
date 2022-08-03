/** ↓ エディタで補完を効かせるための JSDoc */
/** @type {import('webpack').Configuration} */

const webpack = require('webpack');
const html_plugin = require("html-webpack-plugin");

module.exports = {
    mode: "development",
    devtool: "source-map",
    devServer: {
        static: {
            directory: "./dist",
        },
    },
    module: {
        rules: [
            {
                // 拡張子 js のファイル（正規表現）
                test: /\.js$/,
                // ローダーの指定
                loader: "babel-loader",
            },
            {
                test: /\.html$/,
                loader: "html-loader"
            }
        ],
    },
    plugins: [
        // Work around for Buffer is undefined:
        // https://github.com/webpack/changelog-v5/issues/10
        new webpack.ProvidePlugin({
            Buffer: ['buffer', 'Buffer'],
        }),
        new html_plugin({ template: "./src/html/index.html" }),
    ],
    resolve: {
        fallback: {
            "stream": require.resolve("stream-browserify"),
            "buffer": require.resolve("buffer")
        }
    },
    experiments: { 'asyncWebAssembly': true }
};