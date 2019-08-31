# 概要
rust, webassemblyをキーワードにブラウザで動作する、テトリスもどきを作成しました。
HTMLでの描画はcanvasです。

# 使い方
1. git cloneしてローカル環境に持ってきます。
2. 下記コマンドでローカル上にサーバを立てます。
```
$ /path/to/bin/up_server.sh
```
3. ブラウザで http://localhost:8000/html/index.html にアクセスします。

# コードの改修方法
## js, htmlの改修
1. html/index.htmlを修正します。
2. ブラウザを再読み込みすれば、反映されます。

## rustの改修
1. lib.rs を修正します。
2. bin/build.sh を実行します。
3. bin/up_server.sh が動作している場合は、再起動します。
4. ブラウザで確認します。

# 開発時のメモ
## 作成方針 
canvasの関数をrustから呼び出すことは難しそう？なので、自前でピクセル操作を行う。

## 画面設計
フィールドの大きさ 10px * 20px
1ブロックの大きさ 22px * 22px
描画開始位置 (20, 20)
NEXTは1つ
スコアは6桁

# 実装メモ
* webassemblyの場合、ヒープは使用できない？
BoxもVecもエラーを吐いた。使い方がよくわかっていない感じ。とりあえず、スタックのみで実装する。
そもそも、static mutを使った実装に問題がありそう。ちゃんとメモリ確保してから操作したほうが良さそう。
第一版は現行で進めて、いつか改めて実装。rocket_wasmではヒープとか使っている。

* デバッグログを出力しようとするとJSエラーを吐く
rust側にconsole_logメソッドを作成したが、js側でエラーが出力。
描画に使用しているメモリ空間に、console_logのテキスト情報が入ってしまったため、エラーとなったみたい。
メモリを再定義し直すことで、JS側のメモリが破壊されても、復旧させることができる？
とりあえず、JS側で描画で使用するメモリ空間の再定義を行うようにしたら、エラーがでなくなった。
参照: https://github.com/emscripten-core/emscripten/issues/6747

* 全体的にrustのプログラムではない
チュートリアルを軽く読んだだけで実装したので、rustの利点を活かしきれていない。
何かの機会で改めて見直す。

# 参考資料
* RustでWebAssemblyを扱う方法  
http://nmi.jp/2018-03-19-WebAssembly-with-Rust  
* jsのrequestAnimationFrameの扱い方
http://yomotsu.net/blog/2013/01/05/fps.html
* Rustコーディングメモ
https://qiita.com/honeytrap15/items/c7a13c7f2640192b6753
* rocket
https://github.com/aochagavia/rocket
* Minecraft4kRust
https://github.com/tkihira/Minecraft4kRust

# ループについて
rustではforよりも、whileやloopを使ったほうがよいことがある。
https://qiita.com/aimof/items/8e710f928c1ffbb1faf0