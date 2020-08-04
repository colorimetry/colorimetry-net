---
layout: default.liquid
title: ColorSwitch App
---

<div id="app-main">
</div>

<noscript>
    <h1>❌ Error: your browser does not support JavaScript, but this page requires it. ❌</h1>
</noscript>
<script nomodule>
    document.body.innerHTML = "<h1>❌ Error: your browser does not support JavaScript modules. ❌</h1>";
</script>
<script type='text/javascript'>
    // Check if WebAssembly is supported. Code from
    // https://stackoverflow.com/questions/47879864 .
    const supported = (() => {
        try {
            if (typeof WebAssembly === "object"
                && typeof WebAssembly.instantiate === "function") {
                const module = new WebAssembly.Module(Uint8Array.of(0x0, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00));
                if (module instanceof WebAssembly.Module)
                    return new WebAssembly.Instance(module) instanceof WebAssembly.Instance;
            }
        } catch (e) {
        }
        return false;
    })();
    if (!supported) {
        document.body.innerHTML = "<h1>❌ Error: your browser does not support WebAssembly. ❌</h1>" +
            "<p>For a list of supported browsers, see <a href=\"https://caniuse.com/#search=WebAssembly\">this</a>.<p>";
    }
</script>
<script src="colorimetry-net.js"></script>
