---
weight: 999
title: リアクション
description: ""
icon: "article"
date: "2025-02-24T02:20:30+09:00"
lastmod: "2025-02-24T02:20:30+09:00"
draft: false
toc: true
---

Misskey や Pleroma などの一部の ActivityPub 実装はノートに対する絵文字のリアクションをサポートしています。
Lightpub では絵文字によるリアクションを実装していないため、絵文字リアクションが行われた場合、"Like" (星マーク) として処理されます。

<!-- なお、絵文字リアクションは今後の Lightpub リリースで実装する予定です。 -->

## Misskey との連合時の挙動

Lightpub 上で Like した場合: Misskey 上では「♥️」リアクションとして表示されます。

Misskey 上で任意の絵文字のリアクションを付与した場合: Lightpub 上では Like (☆) として扱われます。
