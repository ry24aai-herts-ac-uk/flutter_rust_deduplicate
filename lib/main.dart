import 'dart:convert';
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:file_picker/file_picker.dart';
import 'package:ffi/ffi.dart';
import 'blake3_ffi.dart';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Blake3 Hash',
      theme: ThemeData(
        primarySwatch: Colors.blue,
      ),
      home: const MyHomePage(),
    );
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({super.key});

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  final List<DataRow> _hashRows = [];

  Future<void> _getHashesForDirectory() async {
    final result = await FilePicker.platform.getDirectoryPath();
    if (result != null) {
      final directory = Directory(result);
      final files = directory.listSync(recursive: true);
      for (final file in files) {
        if (file is File) {
          final path = file.path;
          if (path.endsWith('.jpg') ||
              path.endsWith('.jpeg') ||
              path.endsWith('.png') ||
              path.endsWith('.gif')) {
            final pathPtr = path.toNativeUtf8();
            final resultPtr = blake3HashFile(pathPtr);
            final resultJson = resultPtr.toDartString();
            freeString(resultPtr.cast());
            final resultData = jsonDecode(resultJson);
            malloc.free(pathPtr);
            setState(() {
              _hashRows.add(DataRow(cells: [
                DataCell(Text(resultData['file_name'])),
                DataCell(Text(resultData['file_path'])),
                DataCell(Text(resultData['blake3_hash'])),
                DataCell(Text(resultData['is_duplicate'].toString())),
              ]));
            });
          }
        }
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Blake3 Hash'),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[
            Expanded(
              child: SingleChildScrollView(
                scrollDirection: Axis.horizontal,
                child: DataTable(
                  columns: const [
                    DataColumn(label: Text('File Name')),
                    DataColumn(label: Text('Path')),
                    DataColumn(label: Text('Blake3 Hash')),
                    DataColumn(label: Text('Is Duplicate')),
                  ],
                  rows: _hashRows,
                ),
              ),
            ),
          ],
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _getHashesForDirectory,
        tooltip: 'Get Blake3 Hashes for Directory',
        child: const Icon(Icons.folder),
      ),
    );
  }
}