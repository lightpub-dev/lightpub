---
weight: 999
title: インストール
description: インストール手順の解説です。
icon: "article"
date: "2025-02-24T01:08:18+09:00"
lastmod: "2025-02-24T01:08:18+09:00"
draft: false
toc: true
---

大きく分けて2通りのインストール方法があります。

1. Docker compose を使用してインストール (推奨)
2. ソースからビルドしてインストール

## Docker compose を使用したインストール手順

1. Docker と git をインストールします。
2. https://github.com/tinaxd/lightpub_rs から git clone します。
3. `cd lightpub_rs`
4. `docker-compose.yml` を編集します。特に次の環境変数が重要です。
    - SESSION_KEY: クッキーの暗号化に使用します。ランダムな文字列を設定してください。
    - REGISTRATION_OPEN: `true` ならばユーザーの新規登録を有効にします。ユーザーを登録する予定がない場合は `false` にしてください。
    - LP_BASE_URL: Lightpub をホストする URL を設定します。**一度設定したら変更しないでください。** 変更すると他サーバーとの連合が正常に動作しなくなる可能性があります。
    - DEV_MODE: `true` ならば開発環境モードを有効化します。開発環境モードではセキュリティ上の問題があるため、**プロダクション環境では必ず `false` に設定してください。**
5. `docker compose up -d`

## ソースからビルドしてインストール

Postgresql と Redis を手動でインストールする必要があります。

準備中。
