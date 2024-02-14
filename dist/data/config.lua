-- -*- Mode: Lua; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2; coding: utf-8-unix; -*-
-- 起動時に静的決まる内容だけ、ここに記述する
-- $Id: config.lua 14047 2008-07-17 14:07:20Z ashishido $

-- =========================================================================================================
-- global variable
TRUE  = 1
FALSE = 0


-- =========================================================================================================
-- 垂直同期を取るかどうか添字が文字列なのは、もしかすると、添字自体を取得できたら、表示をループで終わらせる事が可能となる為
Config = {}



-- =========================================================================================================
-- 起動時一度だけ参照される
-- 垂直同期を取るかどうか。描画時間を計測したい場合、これをFALSEにする。でないと、VSYNC待ちが発生し、一見すると最大量の処理時間が取られていると錯覚する
Config[ "VSYNC" ] = TRUE

-- JAMMAでバイスを使用するかどうか
Config[ "JAMMA" ] = TRUE

-- 竹中さんが使用する背景当たり表示用サービスを生成する
Config[ "DRAWPRIMITIVE" ] = FALSE

-- スクリーンショットを取れる様にする
Config[ "SCREEN-SHOT" ] = FALSE

-- ストリームを再生しない用にする
Config[ "STREAM-STOP" ] = FALSE

-- ライバルカーを非表示にする
-- -1 で通常表示。3->2->1->0の数値でrival3>rival2>rival1>my>で非表示になる
INVISIBLE_CAR = -1

-- 画面解像度(masterは640.480)です 1280.1024
-- パブ用
--SCREEN_XSIZE = 1280
--SCREEN_YSIZE = 1024
SCREEN_XSIZE = 640
SCREEN_YSIZE = 480

-- レースシーンFSAA
RACE_FSAA = FALSE

-- =========================================================================================================
-- シーケンスに入る際に参照される
-- パフォーマンスチェックを起動するか。seqmmiyoshiで性能チェック用ダミーオブジェクトが表示される
Config[ "PERFORMANCE" ] = FALSE

-- ポストエフェクトを消す
Config[ "POST-EFFECT" ] = FALSE


-- =========================================================================================================
-- ture/falseを文字列化して返す
function changeBoolToString( hbool )
   local ret_string = "FALSE"
   if hbool == TRUE then ret_string = "TRUE" end

   return ret_string
end

-- 変数取得
function VariableGet( hname )
   if Config[ hname ] == nil then return( 0 ) end
   return Config[ hname ]
end

-- 設定内容を表示
-- alcheymにはprintf等の標準出力のバインドは無い様です。ですので、以下は動作しません
-- が、呼ばれなければアクションが起きないので、記述は残しておきます
function Disp()
   print( "VSYNC"           , changeBoolToString( VariableGet( "VSYNC" ) ) )
   print( "JAMMA"           , changeBoolToString( VariableGet( "JAMMA" ) ) )
   print( "PERFORMANCE"     , changeBoolToString( VariableGet( "PERFORMANCE" ) ) )
   print( "POST-EFFECT"     , changeBoolToString( VariableGet( "POST-EFFECT" ) ) )
end

Disp()
