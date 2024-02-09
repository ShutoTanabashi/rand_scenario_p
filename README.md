# rand_scenario

Rustによる乱数列生成プログラム  

## 製作者

Author: Shuto Tanabashi / 棚橋秀斗  
Mail: [tanabashi@s.okayama-u.ac.jp](tanabashi@s.okayama-u.ac.jp)  

## 注意事項

本プログラムを実行あるいは流用したことにより生じる一切の事象について製作者は責任を負いません。
自己責任でお願いいたします。

## 実行方法

シェルを開きます。
（WindowsならPowerShellやWindows Terminal。macOSやLinuxならzsh, bash, ターミナル。）

cdコマンド等を使用して`rand_scenrio_p`ディレクトリまで移動します。

次のコマンドでこのプロジェクトのドキュメントが読めます。  
注：rustをインストールしていない場合は[rustup](https://www.rust-lang.org/ja/tools/install)をインストールしてください。

```zsh
cargo doc --no-deps --open
```

次のコマンドで実行できます。引数の後ろ3つは適宜変更してください。

```zsh
cargo run --release ./test/test_scenario.toml ./rands 1000
```

ちなみに，引数の後ろ3つは「シナリオを描いたtomlファイル 計算結果の出力先ディレクトリ 生成するファイル数」です。
