import 'dart:ffi';
import 'dart:io';
import 'package:ffi/ffi.dart';

typedef Blake3HashFileC = Pointer<Utf8> Function(Pointer<Utf8> path);
typedef Blake3HashFileDart = Pointer<Utf8> Function(Pointer<Utf8> path);

typedef FreeStringC = Void Function(Pointer<Char> s);
typedef FreeStringDart = void Function(Pointer<Char> s);

final DynamicLibrary _nativeLib = Platform.isAndroid
    ? DynamicLibrary.open('libblake3_hash.so')
    : DynamicLibrary.open('lib/native/libblake3_hash.so');

final blake3HashFile = _nativeLib
    .lookup<NativeFunction<Pointer<Utf8> Function(Pointer<Utf8>)>>('blake3_hash_file')
    .asFunction<Pointer<Utf8> Function(Pointer<Utf8>)>();

final freeString = _nativeLib
    .lookup<NativeFunction<FreeStringC>>('free_string')
    .asFunction<FreeStringDart>();
