# 開発メモ
## 作成方針 
canvasの関数をrustから呼び出すことは難しそう（）なので、自前でピクセル操作を行う。
画像の読み込みに関しても、自前ライブラリを作成して取り込む
## 画面設計
フィールドの大きさ 10 * 20
1ブロックの大きさ 22 * 22
描画開始位置 (20, 20)
中央線 40
NEXTは1つ
スコアは6桁

# 実装メモ
webassemblyの場合、ヒープは使用できない？
BoxもVecもエラーを吐いた。とりあえず、スタックのみで実装する。
そもそも、static mutを使った実装に問題がありそう。ちゃんとメモリ確保してから操作したほうが良さそう。
第一版は現行で進めて、後日改めて実装。ちなみに、rocket_wasmでもstaticは使用していない。
なので、実装し直す。

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