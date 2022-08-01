/** ↓ エディタで補完を効かせるための JSDoc */
/** @type {import('webpack').Configuration} */

const webpack = require('webpack');

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
        ],
    },
    plugins: [
        // Work around for Buffer is undefined:
        // https://github.com/webpack/changelog-v5/issues/10
        new webpack.ProvidePlugin({
            Buffer: ['buffer', 'Buffer'],
        }),
        // new webpack.ProvidePlugin({
        //     process: 'process/browser',
        // }),
    ],
    resolve: {
        fallback: {
            "stream": require.resolve("stream-browserify"),
            "buffer": require.resolve("buffer")
        }
    },
    externals: {
        'wasmer_wasi_js_bg.wasm': true
    },
    experiments: { 'asyncWebAssembly': true }
};