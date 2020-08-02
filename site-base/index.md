---
layout: default.liquid
title: Welcome
---

Welcome to colorimetry.net.

{% for post in collections.posts.pages %}
#### {{post.title}}

[{{ post.title }}]({{ post.permalink }})
{% endfor %}
