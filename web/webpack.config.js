/** ↓ エディタで補完を効かせるための JSDoc */
/** @type {import('webpack').Configuration} */

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
        new html_plugin({ template: "./src/html/index.html" }),
    ],
    experiments: { 'asyncWebAssembly': true }
};