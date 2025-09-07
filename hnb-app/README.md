# hnb-app - automatic colorimetric analysis of HNB dye

This directory contains the source code for [HNB
App](https://colorimetry.net/hnb-app).

## About

This app is based on the [Yew framework](https://yew.rs/docs/).

### Prerequistes for development

1) [rust](https://rustup.rs/)
2) [trunk](https://trunkrs.dev)

### 🛠️ Build

When building for the first time, ensure to install dependencies first.

```
trunk release
```

### 🔬 Serve locally

```
trunk serve --open
```

To view the CSS in local development, additionally do the the following
(unfortunately, this does not hot load changes to CSS).

```
cd site-back
cobalt build
cp _site/style.css ../hnb-app/dist
```
