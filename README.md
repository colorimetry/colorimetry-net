# colorimetry.net

## About

This repository houses the source code for https://colorimetry.net/.

- `hnb-app` directory has the web app written in the rust language.
- `site-base` has the source code for the website using a static site generator.

## Development process

Pushes to the main branch on Github will automatically go to production within a
few minutes due to Netlify. Netlify builds other branches and developers with
correct permissions can see [all builds for all
branches](https://app.netlify.com/sites/colorimetry/deploys).

## Code of conduct

Anyone who interacts with this software in any space, including but not limited
to this GitHub repository, must follow our [code of
conduct](code_of_conduct.md).

## License

MIT / Apache 2.0

See `LICENSE_MIT` or `LICENSE_APACHE`. Take your pick.

## 🛠️ Build

```
# On linux:
build.sh
```
<a href="https://www.netlify.com"><img src="https://www.netlify.com/img/global/badges/netlify-color-bg.svg" alt="Deploys by Netlify" /></a>

Using docker, this builds all output as static files into the `dist` directory
in the docker image, which will be tagged as `colorimetry-net:latest`:

```
docker build -t colorimetry-net:latest .
```
