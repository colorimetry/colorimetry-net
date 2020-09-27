# hnb-app - automatic colorimetric analysis of HNB dye

This directory contains the source code for [HNB
App](https://colorimetry.net/hnb-app).

## About

This app is based on the [Yew framework](https://yew.rs/docs/) and the source
code repository was forked from the [Yew Webpack
Template](https://github.com/yewstack/yew-wasm-pack-template).

### 🛠️ Build

When building for the first time, ensure to install dependencies first.

```
yarn install
```

```
yarn run build
```

### 🔬 Serve locally

```
yarn run start:dev
```

To view the CSS in local development, additionally do the the following
(unfortunately, this does not hot load changes to CSS).

```
cd site-back
cobalt build
cp _site/style.css ../hnb-app/dist
```
